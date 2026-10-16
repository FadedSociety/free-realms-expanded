# 022 House loading and visiting

Status: entrance/zone/instance packets recovered from a preserved live-session capture and
re-encoded byte for byte; a bounded server-side instance router with owner/guest routing is
implemented and green (12 Rust tests + 10 Python checks). No native client has been pointed at this
code, so the native gate is not claimed.

Owned paths: `prototype/osfr-tools/src/restoration/piece_022.rs`,
`prototype/osfr-tools/tests/piece_022.rs`, `scripts/pieces/piece_022.py`,
`tests/pieces/test_piece_022.py`, this note, and
`lab/reports/piece-022-20261009T025254Z/`. No shared file, manifest, gateway, launcher, content,
state or release path was modified.

## Original evidence

The capture is a real client session preserved by the OpenSourceFreeRealms community:

| File | sha256 |
| --- | --- |
| `lab/references/OpenSourceFreeRealms/ofrserver/Packets/p2.pcap` | `65ac0801bf49fa9af4ee0bc95568b751392ec4d594fe6ac4e89605f9ee0d0108` |
| `p12.pcap` (same session plus a second one) | `99309eb5d3287bb458bf3e79332efcfc7ded3ba742077bb7258ab34a8addfe97` |
| `packets_2.pcapng` (same session) | `221b8d548d0c01bea5f84bb0eadece91e7902ec0c81ffddd989cdea6e0775f37` |

Decoding method: SOE opcode-9/13/21 reliable channel, `0x0019` multi-message containers, and the
two-byte seeded CRC trailer recovered from the server direction (26,657 of 26,662 server frames
validate; the rest are recorder-truncated). The capture also contains plain HTTP patch fetches; the
game session runs between `192.168.1.145:61759` and `64.37.170.6:20267`. That public endpoint sits
in the SOE address range and the same client fetched live patch assets, so this is a live-client
capture; calling it the original production server specifically is an inference, not a proven fact.

Recovered sequence (server frame numbers are the capture's):

| Frame | Direction | Message |
| --- | --- | --- |
| 27110 | client | `007f 0017` — `ClientHousingPacketRequestGrant` (127/23), empty body |
| 27195 | client | BrowserV2 `SelectHouse 573418857690894554` plus `007f 0013` — `ClientHousingPacketEnterRequest` (127/19) |
| 27311 | server | inner opcode 31 `ClientUpdatePacketBeginZoning` to `hsg_emptylot_seaside_beach_01` |
| 29881 | server | `007f 002d` — `HousingPacketZoneData` (127/45), second flag set |
| 31311 | server | second 127/45, same instance data, second flag clear |
| 31507+ | server | nine `007f 0028` — `HousingPacketFixtureUpdate` (127/40) messages |

Exact bytes (also embedded in the Rust test with these hashes):

| Vector | Length | sha256 |
| --- | --- | --- |
| `7f001700` | 4 | `21759f50d68355847fde34569f13761327727d1b0e225feffb48dfe38e4b8bbb` |
| `7f001300da102db86931f5070000000000` | 17 | `f41afd28fc71267d09fe252b6ffed2b4a30fcc963db51175272e731c49d60205` |
| BeginZoning (see checker output) | 108 | `f3b03575443e99b5d3bd15f24752eedeec258375ac0d995a876a78cb5a435f07` |
| ZoneData first (second flag set) | 118 | `24a4e13793bab260e7eedcb293d5715c642a07790a50befaa5028792cb816200` |
| ZoneData later (second flag clear) | 118 | `d00b1b4ba137911bb123e99116295b8f3a6c0ca5f44ddb915117afc6d4831ea6` |

Field facts read out of those bytes:

* `ClientHousingPacketEnterRequest` body = `u64 house_guid`, `i32 unknown`, `bool unknown2`. The
  captured guid is `0x07f53169b82d10da` = `573418857690894554`, which is also the decimal string in
  the BrowserV2 `SelectHouse` message that precedes it.
* BeginZoning = string zone name, `i32 type` (2), position `f32 x4`, rotation `f32 x4`, string sky,
  `bool tutorial`, `u8 unknown`, `i32 id`, `f32 update radius`, `i32 geometry id`,
  `bool override update radius`, `bool wait for zone ready`. Captured values: zone
  `hsg_emptylot_seaside_beach_01`, id 1, geometry 214, position (440.632, -0.071, 432.801),
  rotation (-0.9999741, 0, -0.0072035603, 0), sky `sky_seaside24.xml`, unknown 1, override true,
  wait false.
* `HousingPacketZoneData` = `bool IsPreview`, `bool Unused`, `i32 HeadSize`, then the community
  `PlayerHousingInstanceInfo` order. Captured: both `IsPreview = false`, `HeadSize = 10`, owner
  guid `0x4b52cc6a9573ca91` = `5427625259080862353`, instance guid = the requested house guid,
  name id 8321, house name `{n1} Cool Castle`, owner name `Ma\u00ffhem`, icon 33203, fixture count 0,
  furniture score 0, flora allowed. Both 118-byte messages decode to their last byte with the
  community field order, and the second flag differs between them.
* The nine fixture updates carry fixture guid `0x07f54ca17bf5c001` first and house guid
  `0x07f53169b82d10da` second, so furniture binds to the same instance identity the entrance used.

Independent agreement, three sources:

1. the supplied server's `LibSOE/SOE/BaseHousingPackets.cs` maps 19/23/40/45 into base 127
   (`BasePackets.cs` line for `BaseHousingPacket = 127`);
2. the native client builds the same family (`FUN_00aa83c0` = `FUN_00aa4990(0x13)` then
   `ClientHousingPacketEnterRequest::vftable` with the guid at +0x10, int at +0x18, bool at +0x1c;
   `FUN_00ab07a0` = 0x2d `HousingPacketZoneData` with `HeadSize` default 10 in
   `lab/decompiled/client/FreeRealms/pseudocode.c`);
3. the pinned community gateway's commented test vector for this lot quotes the same numbers as the
   captured BeginZoning packet, so that comment is a copy of this capture rather than an invention.

Supporting original data: `Houses.json` row 1 (seaside) has `ZoneId` 1, matching the captured
BeginZoning id; the client asset catalog holds 68 `hsg_emptylot_seaside_beach_01` terrain entries.

## Recovered community facts (not original binaries)

* Housing zone instances are keyed by `(zone definition id, owner id)` and refuse a null owner
  (`Sanctuary/src/Sanctuary.Game/ZoneManager.cs`), so an owner and a guest resolve to one instance.
* `ZoneManager.EvictIfEmpty` disposes an empty non-starting zone; the next entrance recreates it with
  a fresh runtime id while the house data behind it is unchanged.
* `PlayerHousingInstanceInfo` and `HousingPacketZoneData` field order; the two captured responses
  re-encode from that model byte for byte.
* The wiki corroborates guest visiting at design level: "Your friends can visit your house and help
  you keep your plant watered" ([Jackpot Plant](https://freerealms.fandom.com/wiki/Jackpot_Plant),
  revision 44066) and lots as owned housing areas ([Lots](https://freerealms.fandom.com/wiki/Lots),
  revision 49783).

## What this piece reconstructs

`HousingEntranceRouter` (owned module) keeps the persisted house table and routes entrances:

* runtime instance ids are local and monotonic (first value 1, mirroring the community counter);
* the `Unused` flag is set on the first zone-data response after an instance is created and cleared
  after, which matches the two captured responses; the reason is unrecovered;
* the request's `unknown` int is treated as the preview flag because the community model comments
  "Set by EnterPreview" and the captured pair is (0, `IsPreview = false`). This is an inference;
* one `(zone definition, owner)` pair maps to one house; a second registration for the same pair is
  refused instead of silently splitting an owner's instance;
* the snapshot format is local (`schema_version` 1, houses only). Runtime instances are deliberately
  not restored: the community server recreates zones on demand, so a restart starts with none and
  owners and guests re-converge on the same house identity;
* no permission or invitation policy. Denials, locks and `HousingPacketHousingError` (127/46) belong
  to piece 023; this module returns typed errors and never fabricates a denial packet.

## Deliberate gaps

1. No guest-visit capture exists. The only housing entrance captured is a single player entering
   their own lot (instance guid = entered house guid, owner guid different from the instance). The
   guest path is therefore recovered-community plus reconstructed, not observed.
2. The captured session's second connection (`p12.pcap` flow 0/1) contains no housing messages, so
   there is no second-instance observation and no instance-eviction observation.
3. `HousingPacketHousingError` (46) has no recovered body here; piece 023 encodes a proposal whose
   native acceptance is still open.
4. The 127/45 tail word observed as `0x00130000` (community slot name "Votes") is preserved
   verbatim. Its meaning is unknown; it is not treated as a vote count.
5. The client-to-server CRC/trailer model was not recovered with the server model (client frames do
   not validate under it). Client messages are decoded structurally from their own length fields, so
   the enter-request bytes are exact even though that frame's checksum model is open.
6. No native acceptance: the Rust router has not answered a real client. Entering a lot, seeing the
   house data, and being refused for an unknown house all still need the lead's gateway integration.

## Verification

Commands and results (full logs in the report directory):

* RED: `scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_022`
  failed with `error: couldn't find file tests\..\src\restoration\piece_022.rs` before the module
  existed.
* GREEN: same command → `12 passed; 0 failed`, including byte-exact re-encoding of the captured
  BeginZoning and both ZoneData bodies, owner/guest instance sharing, eviction, restart and malformed
  input rejection.
* `..\lab\tools\wiki-env\Scripts\python.exe -m unittest discover -s tests/pieces -p
  "test_piece_022.py" -v` → `Ran 10 tests ... OK`, including the cross-check that the Rust vectors
  equal the bytes re-derived from `p2.pcap`.
* `python scripts/pieces/piece_022.py` → `"checks": "all passed"` and the four vector groups above.
* The ordinary full suite currently fails to compile because other workers' in-flight files
  (`tests/piece_030.rs` and its missing module) are present; that is outside this item. The targeted
  piece_022 target and the Python tests are green.

## Next gates

1. Lead integration: register `pub mod piece_022;` (or move the file) and drop the `#[path]` include.
2. Wire integration: answer 127/19 with 31 BeginZoning + 127/45 zone data and route the player into
   the instance; reuse `encounter::zone_begin_packet`'s ordering (this module's encoder already
   matches it, including the extra `Unknown`/`WaitForZoneReady` bytes).
3. Property source: feed `HouseRecord.zone_definition_id/house_guid/owner_guid` from pieces 021/006
   instead of a test fixture.
4. Guest acceptance needs a real second client; permission refusal needs piece 023's denial packet.

Cross-links: [[Housing Restoration]], [[../Design/MMO Worker Contract]], [[Wiki Research Database]],
[[Rust Gateway Milestone]].
