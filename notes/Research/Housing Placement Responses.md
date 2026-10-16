# Housing placement responses

Read-only recovery of the housing placement response bodies, 2026-10-09 (UTC). Reviewer owns this
note and `lab/reports/housing-placement-responses-20261009T1705Z/` only; no src/lib/gameplay/test/
content/save/UI/process/release file was changed. Skills: RTK (normal commands), Ponytail (one
decoder reusing the piece-022 transport tool, one small oracle), Caveman (terse report),
reverse-engineering (the client reader anchors are decompile claims; read as such), game research
(pinned community sources first).

**Verdict: the captured placement flow is recovered with bounded bodies for 127/2 (client to
server), 127/40 (server to client, fixture record) and 127/41 (server to client, remove), plus a
bounded client-reader layout for 127/53 (placement denied, no capture vector). The briefing's
"S2C 127/2 PlaceFixture" does not match this capture: sub 2 is client-to-server (8 sends), and the
placement reply is sub 40 (9 records) with a 127/44 house-info update. No native acceptance and no
unknown bytes are serialized.**

## Evidence base

| Input | sha256 |
| --- | --- |
| `lab/references/OpenSourceFreeRealms/ofrserver/Packets/p2.pcap` | `65ac0801bf49fa9af4ee0bc95568b751392ec4d594fe6ac4e89605f9ee0d0108` |
| `lab/decompiled/server/LibSOE/SOE/BaseHousingPackets.cs` | `93d0cfa1d93ebc12fa2386063cb75d19ba8053a627d026569ea786d4b083b1c4` |
| `lab/decompiled/client/FreeRealms/pseudocode.c` | `f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3` |
| Sanctuary `FixtureInstance.cs` / `PacketWriter.cs` / `PacketReader.cs` / `CustomizationDetail.cs` / `ISerializableType.cs` / `IDeserializableType.cs` | copied byte-identically into the private report (`oracle/src/`, hashes in `final-hashes.txt`) |

Native anchors read for this note: constructors `ClientHousingPacketPlaceFixture` (`FUN_00aa53f0`,
pseudocode `1384396-1417`), `HousingPacketFixtureRemove` (`FUN_00aa80d0`, `1386037-6054`),
`HousingPacketFixtureUpdate` (`FUN_00aad200`, `1390129-1558`),
`HousingPacketFixturePlacementDenied` (`FUN_00aad660`, `1400401-1422`); handlers `1400419` (53),
`1401572` (41), `1409746` (40); client build/send sites `1405232`/`1409713` (2); string reader
`FUN_00ab3030` (`1395417`); denial path `1400448-1458` ("Fixture placement denied." plus the
`HousingMaxFixtures` property). The decompiled server build contains only the housing enum, so the
community serializer is the only server-side writer reference for 40.

## Captured flow (frame, direction, sub, length)

`31494 c2s 2 (42)` -> `31507 s2c 40 (229)` + `44`; seven further `2 -> 40 + 44` pairs
(`31914/31919`, `32016/32023`, `32194/32196`, `32275/32282`, `32335/32342`, `32574/32578`,
`32619/32623`, plus one unpaired `40` at `32098`); `1 -> 42` twelve times (fixture asset preview);
`5 -> 51` (save -> update position); `3 -> 41` (pickup -> remove); `45` zone data on entry; `53`
never occurs (0 of the 81 inner-127 messages).

## Recovered bounded layouts

**127/2 `ClientHousingPacketPlaceFixture` (client to server, 8 captured records, 42 bytes;
`u16 opcode` + 40-byte body):**

| Body offset | Field | Provenance |
| --- | --- | --- |
| 0x00 | `u32 first_dword` (captured 4260, 4260, 4378, 4179, 4214, 4594, 3996, 4595) | capture; identity **not established** (not the reply's fixture definition id 34; 4260 is reused for two placements) |
| 0x04 | `f32[4] position` (w = 1.0; every value inside the `Houses.json` property-1 build area) | capture, layout from the client build site |
| 0x14 | `f32[4] rotation` (captured `(0,0,0,0)` in all 8) | capture; semantics unknown |
| 0x24 | `f32 scale` (1.0 in all 8) | capture |

Each captured 2 is answered by a 40 whose position equals the requested position exactly, and the
fixture guids increment `0x07f54ca17bf5c001..c009`.

**127/40 `HousingPacketFixtureUpdate` (server to client, 9 captured records, 229 bytes;
`u16 opcode` + 227-byte payload):** the first 133 bytes are the community `FixtureInstance` record
order (guid, house guid, fixture definition id 34, `unknown4` = 0, position, rotation, tilt, npc
guid, tint id, `unknown10`, customization `{type, texture alias, tint alias, tint id, texture
override}`, three strings, xml data, scale 1.0, bool, `unknown17`). The unchanged Sanctuary
`FixtureInstance.Serialize` reproduces that 133-byte prefix **byte-identically for all 9 records**
(`oracle.log`). The remaining **94 bytes are not decoded** (unknown): they begin with the same
fixture guid at +133, contain further ids (e.g. 7466 and 34) and two 1.0 floats, and are distinct
per record. A complete reply cannot be claimed until they are decoded; whether the client reader
requires them is not established.

**127/41 `HousingPacketFixtureRemove` (server to client, 8 captured records, 10 bytes;
`u16 opcode` + 8-byte body):** one `u64 fixture guid`. Every captured removal guid
(`c001..c009` minus `c003`) appears in the 40 update set; all are unique. The client handler looks
the guid up and removes that fixture.

**127/53 `HousingPacketFixturePlacementDenied` (server to client, no capture vector):** the client
handler constructs the packet and reads one length-prefixed string (`i32 byte_length` + UTF-8,
reader `FUN_00ab3030`); if the string is empty it falls back to the local `HousingMaxFixtures`
property and logs "Fixture placement denied.". The bounded layout is that string. Any further
field is not read by this path and is not established; a synthetic vector is included.

## Synthetic vectors (encoded from the recovered layouts)

`synthetic-vectors.json` + `vectors/synthetic-*.bin`: sub 2 (`0200` + 4260 + the captured
frame-31507 transform + zero rotation + 1.0), sub 41 (`2900` + `0x07f54ca17bf5c009`), sub 53
(`3500` + length 25 + "Fixture placement denied."). The sub-2 vector is byte-identical to the
captured frame-31494 body, as expected from the reused values.

## Checks

* `analyze-placement-responses.py` re-derives every vector from the pinned p2.pcap through the
  piece-022 transport decoder: **91/91 checks pass** (`analyze.log`), covering the capture sha256,
  the 2/40/41 directions and counts (8/9/8), the build-area bound on every transform, the 2->40
  transform echo, the `c0xx` guid cross-link between 40 and 41, and the 94-byte 40 tail per record.
* The oracle (`lab/tools/dotnet/dotnet.exe`, unchanged community serializer) prints
  `oracle: 9/9 fixture prefixes byte-identical` (`oracle.log`).

## Unknowns and limits

The first dword of 2, the rotation semantics, the 94-byte 40 tail, and any field of 53 beyond the
string are unrecovered. The community `FixtureInstance` is a reconstruction used here as an
encoding oracle and validated against this capture; it is not original server source. Nothing here
has been accepted by the native client, and raw capture bytes/decompiled code stay in the lab.

## Minimal next integration scope

1. Serialize the 133-byte 40 fixture prefix plus a 44 house-info update after a placement (oracle-
   proven bytes today); the 94-byte tail stays a blocker until decoded.
2. Answer a pickup with 41 = the server-minted fixture guid; answer a save with 51.
3. Parse 2 as client input (first dword + transform) and map the first dword to the item ledger
   only after its identity is recovered.
4. Use the one-string 53 body for the `HousingMaxFixtures` denial with an accurate reason text.
5. Native acceptance stays with work item 030.

Cross-links: [[Housing Restoration]], [[Pieces/025 Furniture placement client wire]],
[[Pieces/022 House loading and visiting]], [[../WorkItems/025]], [[../WorkItems/030]].

## Deep checkpoint: 127/44 and the 127/2 first dword (2026-10-09, same reviewer)

Preserved previous revision: `lab/reports/housing-placement-responses-deep-20261009T1830Z/note-before-append.md`
is byte-identical to this note before this section (old note sha256 `4AF8A2EF…`);
`previous-receipt.txt` holds the previous report receipt (`14C72810…`). New private report:
`lab/reports/housing-placement-responses-deep-20261009T1830Z/` (`deep-check.py`, `deep-check.log`
91/91 ok, `deep-report.json`, `final-hashes.txt`). No src/tests/content/save/UI edit; still no
native acceptance.

**Count units.** The analyzer/tunnel bodies include the subtype `u16` and exclude base opcode 127:
127/40 = 229 = subtype 2 + fixture record 133 + tail 94 (231 with the base opcode); 127/2 = 42 =
subtype 2 + payload 40 (44 with base); 127/44 = 22 = subtype 2 + payload 20 (24 with base). The
`vectors/*.bin` of the previous report use the 229/42 unit.

**127/44 `HousingPacketUpdateHouseInfo` recovered as a fixed 22-byte record, 19/19 captured
bodies.** Layout: subtype `u16` + `u8` InEditMode, `i32` Unused, `u8` IsLocked, `u8` IsFloraAllowed,
`u8` PetAutospawn, `i32` CurFixtureCount, `i32` CurLandmarkCount, `i32` FurnitureScore (community
`HousingPacketUpdateHouseInfo` field order; native ctor `FUN_00aa5080` = `FUN_00aa4990(0x2c)`,
pseudocode 1384240-1263). Captured values: InEditMode 0 then 1; Unused 10; IsLocked 0;
IsFloraAllowed 1; PetAutospawn 0; CurFixtureCount 2015, 2015, 2016 … 2024, 2023, 2022, 2021, 2020,
**2018**, 2017, 2016, 2015; CurLandmarkCount and FurnitureScore 0. Re-encoding from the parsed
fields is byte-identical 19/19. Semantics label: the field *names* are community-model names and
the ladder is capture-consistent with a live fixture count during placements/removals (the one
observed −2 step 2020→2018 has no own 44 record); this is not native-proven, and the record length
is fixed — only the values vary.

**127/2 first dword — provenance and all-8 table check.** Native: the value is the placement
object's `+0x40` (`FUN_0105bfa0()`), copied into the processor `+0x4160` (pseudocode 1407923-1407936,
1410183-1410190, 1410225-1410229) and emitted as the first payload dword at 1409713-1424. Capture:
`[4260, 4260, 4378, 4179, 4214, 4594, 3996, 4595]` (4260 reused for two placements). All eight
values resolve as Ids in community `ClientItemDefinitions.json` (7 unique), but the resolved entries
are unrelated gear — 3996 Doggy Paper Hat, 4179 Battle Axe, 4214 Warlord Axe, 4260 Elite Black
Shortbow, 4378 Baseball Bat, 4594/4595 Mega Hammer — so the "placed furniture item definition" role
is **not supported**; the dword's identity stays an explicit blocker (it is whatever `+0x40` of the
placement object holds).

**Still open:** the 94-byte 127/40 tail, any 127/53 field beyond the single string, and native
acceptance. Raw capture bytes and decompiled code remain lab-only.
