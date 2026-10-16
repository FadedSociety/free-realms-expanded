# 077 Warpstones and travel

Work item 077 restores the warpstone travel path: the packet a client sends when
the Atlas sends the player to a warpstone, the position/rotation update the
server answers with, the point-of-interest destination list the client builds
its Atlas from, and a server-side gate that refuses destinations the character
has not attuned. The wire layouts are recovered from preserved original
captures and the native client export; the gate is reconstruction and is
labelled as such everywhere it appears.

## Original facts

### First-party native client export

`lab/decompiled/client/FreeRealms/pseudocode.c`
(sha256 `f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3`) binds
every opcode used here. The base id is written into the packet object next to the
vftable, and matches the community `BasePackets` table the legacy server ships
(independent agreement; the table itself is not first-party evidence).

| Packet | Function | Id | Payload the function stores |
| --- | --- | --- | --- |
| `PacketPointOfInterestDefinitionRequest` | `FUN_00b97a20` | `0x38` = 56 | none |
| `PacketPointOfInterestDefinitionReply` | `FUN_00b97ad0` | `0x39` = 57 | none at construction |
| `PacketWorldTeleportRequest` | `FUN_00c1a5e0` | `0x3a` = 58 | two 32-bit arguments (a 64-bit guid) |
| `PacketZoneTeleportRequest` | `FUN_00b97b80` | `0x5a` = 90 | exactly one 32-bit argument |
| `PacketZoneSafeTeleportRequest` | `FUN_00bf7a50` | `0x7a` = 122 | none |
| `ClientUpdatePacketUpdateLocation` | `FUN_0089dd60` | family 38, sub `0xc` = 12 | position/rotation block |

`FUN_00b97c40` is the client's `ClientPointOfInterestManager::InternalDs` column
table and fixes the destination-list field order the Atlas reads:
`Id, Name, Zone ID, X, Y, Z, Heading, IconId, NotificationType, SubName,
BreadcrumbQuestId, AtlasName`.

`FUN_00b991d0` is the client-side sender for `Ui.TeleportToPointOfInterest(id)`:
it looks the id up in the client's own point-of-interest table (`FUN_0102fb50`)
and only then constructs the op-90 packet. The shipped client therefore cannot
address an unknown destination; a modified client still can, which is what the
reconstructed gate below exists for.

### First-party locale rows (`lab/reports/locale-id-to-text.json`)

| Id | Text | Use |
| --- | --- | --- |
| 41300 | "You are now attuned to the Solace Warpstone." | attunement is a real event |
| 24719/24720 | "Warpstones" / "Visit the warpstones and return to them anytime!" | the warpstone collection |
| 24722 | "Visit a Warpstone and return to it any time!" | visit-then-return flow |
| 7199 | "Use Warpstone" | warpstone interaction button |
| 20240, 20830-20832, 24726, 24727, 24730, 24731, 41297, 41298, 41301-41304 | warpstone names | Atlas labels |

### Captured original traffic

The preserved community captures under
`lab/references/OpenSourceFreeRealms/ofrserver/Packets` contain eight real
`PacketZoneTeleportRequest` frames and the `38/12` reply that follows each of
them. `p2.pcap` and `packets_2.pcapng` are the same recording in two container
formats; both are pinned so the checker can keep proving it.

| Capture (sha256) | Request frame | Destination id | Reply frame |
| --- | --- | --- | --- |
| `p2.pcap` `65ac0801…0d0108` | 420 / 1282 | 9 / 155 | 457 / 1302 |
| `p12.pcap` `99309eb5…ddfe97` | 14089 / 14951 | 9 / 155 | 14126 / 14971 |
| `packets_3.pcapng` `c56341ca…fff41a` | 432 / 5393 | 152 / 131 | 456 / 5395 |
| `packets_4.pcapng` `5734593a…8013d5` | 9609 / 10512 | 15 / 31 | 9626 / 10623 |

Recovered layouts, byte-exact in all eight pairs:

* Request, inside the `05`/`06` tunnel after the opcode:
  `i32 destination_id` little-endian, 4 bytes total. Body for id 9 is
  `09 00 00 00`, for id 155 `9b 00 00 00`.
* Reply, after family opcode 38: `u16 sub = 12`, then `f32 position[4]`,
  `f32 rotation[4]`, `bool teleport`, `u8 trailer` (34 bytes after the
  sub-opcode). Captured replies always carry `position[3] = 1.0`,
  `rotation[1] = 0.0`, `rotation[3] = 0.0`, `teleport = 1`; the trailer byte is
  opaque and was observed as 0, 1, 2 and 3.
* The six distinct destinations and the spawn position/rotation the original
  server sent for them are pinned in
  `prototype/osfr-tools/src/restoration/piece_077.rs`
  (`captured_destinations`). They are captured wire values, not a catalogue.

Example, id 9 (`p2.pcap` frame 90 = 420, reply frame 457):

```
request  09 00 00 00
reply    0c 00 | c7 7b 37 c4  42 60 11 41  bc 7c 6c 44  00 00 80 3f
               | df d7 7c 3f  00 00 00 00  7c 50 20 be  00 00 00 00
               | 01 | 01
         position -733.934, 9.086, 945.949, 1.0
         rotation  0.987669, 0.0, -0.156557, 0.0
```

### Community comparators (reconstruction, not first-party)

* `lab/references/Sanctuary` (`PacketZoneTeleportRequest`,
  `PacketZoneSafeTeleportRequest`, `PacketWorldTeleportRequest`,
  `ClientUpdatePacketUpdateLocation`, `PacketPointOfInterestDefinitionReply`,
  `WorldZone.SendPointOfInterests`, `OpenAtlasInteraction`) agrees with the
  captures on the request ids and the `38/12` reply shape and frames the
  point-of-interest list as `true` + record per row, then `false`.
* `lab/decompiled/server` (the legacy build) reads the same op-90 id, answers
  with `SOEWriter(38, isMessage: true)` + `AddHostUInt16(12)`, and ships the
  warpstone and warp-watcher actor placements in `NPCSystem.SendWarpStones` /
  `SendWarpWatchers`. Its reply writes `Heading` and four zero floats where the
  captures carry `position[3] = 1.0` and a non-zero rotation pair, so the
  remaining lanes there are reconstruction, not capture.
* `lab/references/freerealms-wiki` ("Warpstones", "Kellin Warpwatcher") states
  the community rule that a warpstone must be activated before the Atlas offers
  it.

## Reconstructed policy (owned by this item, labelled)

`WarpRegistry` in `piece_077.rs` refuses, without emitting a reply or moving the
character:

* a request body that is not exactly four bytes (`MalformedRequest`),
* an id that is not in the server's destination list (`UnknownDestination`),
* an id that is in the list but not a warpstone (`NotAWarpstone`),
* a warpstone the character has not attuned (`LockedDestination`).

Attunement is modelled as a set the caller fills; the module does not invent
where the server learns it from. The gate is this restoration's policy: the
shipped client already hides unattuned warpstones, and no captured frame proves
the original server ever rejected a locked destination.

## Gaps and remaining gates

* No `56`, `57`, `58` or `122` frame exists anywhere in the fourteen preserved
  captures (`scripts/pieces/piece_077.py` re-checks this). The destination-list
  layout is therefore community-reconstructed on both sides, and whether the
  real `57` body carries the legacy server's length prefix before the record
  stream is unresolved. The Rust decoder tolerates both; the encoder writes the
  bare framed stream.
* No attunement or discovery packet was recovered. How the server learns a
  character attuned a warpstone is unknown; an incremental `57` record is a
  plausible mechanism but is not evidenced.
* Native acceptance has not been run. The lead should, in the shared client:
  use a warpstone and confirm the spawn position/rotation and the trailer byte,
  check which warpstones the Atlas offers, and confirm a locked destination
  produces no movement.
* Integration: register `restoration::piece_077` in `lib.rs`, route the
  tunneled op-90 request through `WarpRegistry`, and source the destination
  table from the recovered `57` catalogue once one is captured or exported.

## Reproducing the evidence

```powershell
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_077.py'
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_077.py" -v
& 'scripts\cargo.ps1' test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_077
```

The private bundle is
`lab/reports/piece-077-20261009T051025Z/`: `warp-scan.json`,
`teleport-windows.json`, `evidence.json`, the two mutation logs and the cargo
logs.
