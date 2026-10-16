# Kart lobby definition framing (102/2)

Read-only recovery, October 9, 2026 (UTC). Scope: recover the complete opcode 102 sub 2 lobby
definition framing and the inner opcode 33 subtype-to-class map from the preserved capture, the
pinned Ghidra export and the pinned community writers, then derive private synthetic serializer
vectors for the future Rust builder. No production source, content, save, UI, release or process
change; no finish, reward, placement or playability behaviour is inferred. Raw captured fixtures stay
in the lab and are not copied into this repository.

Owned artifacts:

| Path | Role |
| --- | --- |
| this note | framing, tables, subtype map, unknowns |
| `lab/reports/kart-lobby-framing-20261009T145825Z/proof.json` | pinned proof, sha256 `da47c4165e6273dce2f3785ab89e8b2589618df7d1aa471deb0820b5730b5b16` |
| `lab/reports/kart-lobby-framing-20261009T145825Z/vectors.json` | private synthetic serializer vectors, sha256 `edfd1fcad7195c83e2e3ec12adb53817b20043f664ff606ccf3ba1eb079025ef` |
| `lab/reports/kart-lobby-framing-20261009T145825Z/trace-kart-lobby-framing.py` | reproducer (re-runnable, `write_exact` outputs) |
| `lab/reports/kart-lobby-framing-20261009T145825Z/trace-run.log` | run output |

Pinned inputs: capture `racing-23867-flow0-frame44523-op102-sub2.bin` and its duplicate
`racing-13231-flow0-frame15239-op102-sub2.bin` (bodies byte-identical), the pcapng, the Ghidra export
`f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3`, and the pinned community
`LobbyGameDefinition.cs` + `LobbyGameDefinitionPacketDefinitionsRequestHandler.cs`; the proof records
the sha256 of each.

## Byte-exact framing

Recovered from the native handler `FUN_0091e060` and the two readers it calls:

```
tunnel header   u16 outer (5 or 6) | u8 flag | u32 inner_length
inner packet    u16 opcode = 102 | u16 sub = 2 | i32 body_length | body[body_length]
relation        inner_length = 8 + body_length
```

* `FUN_0091e060` builds a reader over the inner buffer and calls `FUN_008d67a0`, which reads **two
  i16** values (opcode then sub) and rejects the packet unless the second equals `2`.
* It then calls `FUN_008f74c0`, which re-reads the same two header words and calls `FUN_008bfc00`.
* `FUN_008bfc00` reads the **i32 body length**, checks `cursor + length <= end`, and returns
  `(payload pointer, length)`; truncation sets the reader's error flag and zeroes both outputs.
* The body is then parsed by `FUN_00919b60(reader, client->+0x110)`.

The captured packet: opcode 102, sub 2, body length **2378**, inner length **2386**, body sha256
`51c8f011c781fe376496c851889380ba36f6bbed911daa60eb964ecc71fd5a0c`. Both captured 102/2 frames carry
the same body.

## Body: five tables

`FUN_00919b60` dispatches five tables in this fixed order:

| # | Table | Parser chain | Row layout | Captured rows | Start offset | Bytes |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | games | `008e5450` → `008cd0f0` | `u32 key, i32 x4` (20 B) | 6 | 0 | 124 |
| 2 | options | `008e5580` → `008cd1d0` | `u32 key, i32 x3` (16 B) | 30 | 124 | 484 |
| 3 | values | `008e56c0` → `008cd280` | `u32 key, i32 x3` (16 B) | 43 | 608 | 692 |
| 4 | display order | `0090a070` → `008e8460` | `u32 key, i32, i32, i32 length, length bytes` | 37 | 1300 | 666 (incl. 70 blob bytes) |
| 5 | settings | `008e5800` → `008cd360` | `u32 key, i32 x5` (24 B) | 17 | 1966 | 412 |

Each table begins with an `i32 count`; every row begins with a `u32` key. Display-order rows store
the locale id, then a length-prefixed byte string; the captured rows carry single ASCII digits
(`'2'`, `'4'`, `'6'`, `'1'`, `'3'`, `'5'`) whose locale ids 3032/3033/3034/3036/3037/3040 resolve to
the same digits. Settings rows 1-8 resolve their name id at row[3]: 3031 "Max Players", 3035
"Difficulty", 3038 "Track", 3039 "Required Profile Type", 3040 "Vehicle Race", 3041 "Enable AI",
3042 "Laps".

**Byte-exact proof:** decoding the captured body with this layout consumes **2378 of 2378 bytes** and
the per-table budget above sums to exactly the declared length. The captured games table is:

| key | Id | Type | field4 | NameId | locale |
| --- | --- | --- | --- | --- | --- |
| 1 | 1 | 1 | 37 | 3030 | "Test Lobby Game" |
| 2 | 2 | 2 | 0 | 0 | - |
| 3 | 3 | 3 | 0 | 0 | - |
| 4 | 4 | 4 | 65 | 0 | - |
| 5 | 5 | 5 | 68 | 0 | - |
| 6 | 6 | 6 | 60 | 417078 | "Ship Battle Lobby" |

That catalogue is a **test catalogue** (row 1 is literally "Test Lobby Game", the same row the
pinned community handler hardcodes), so it is framing evidence, not a retail catalogue. A real Kart
Race row would use Type 1 with a real name id such as 3052 "Kart Race Lobby" / 3053 "Demo Derby
Lobby".

The client parser does **not** require exact consumption: a truncated table sets the error flag and
stops, and trailing bytes after the last table are ignored. The exact-consumption result proves the
model above matches the capture; it is not a client requirement.

## Inner 33 subtype-to-class map

Every `VehicleRacePacket*` constructor calls `FUN_00b992d0(<subtype>)` (the family base ctor with
opcode `0x21`) and stores its vftable; extracting that constant gives all 34 subtypes with no gaps:

| Subtype | Class | Subtype | Class |
| --- | --- | --- | --- |
| 1 | VehicleRacePacketUpdateVehicleGameState | 18 | VehicleRacePacketPickupHitResponse |
| 2 | VehicleRacePacketUpdateVehicleGamePlayerState | 19 | VehicleRacePacketUseItemRequest |
| 3 | VehicleRacePacketUpdatePlayerCount | 20 | VehicleRacePacketUseItemResponse |
| 4 | VehicleRacePacketUpdateNpcCount | 21 | VehicleRacePacketItemExpired |
| 5 | VehicleRacePacketUpdateVehicleRaceClientData | 22 | VehicleRacePacketBoostAwardRequest |
| 6 | VehicleRacePacketUpdateVehicleRaceServerData | 23 | VehicleRacePacketBoostAwardResponse |
| 7 | VehicleRacePacketGateHit | 24 | VehicleRacePacketCreateProxiedVehicleRaceMissile |
| 8 | VehicleRacePacketVehicleHit | 25 | VehicleRacePacketDestroyProxiedVehicleRaceMissile |
| 9 | VehicleRacePacketRegisterVehicleRacePlayer | 26 | VehicleRacePacketUpdateVehicleRaceMissileData |
| 10 | VehicleRacePacketVehicleRaceCountdownStatus | 27 | VehicleRacePacketCreateProxiedVehicleRaceExplosion |
| 11 | VehicleRacePacketCreateProxiedVehicleRace | 28 | VehicleRacePacketDestroyProxiedVehicleRaceExplosion |
| 12 | VehicleRacePacketDestroyProxiedVehicleRace | 29 | VehicleRacePacketCreateProxiedVehicleRaceMine |
| 13 | VehicleRacePacketKickProxiedVehicleRace | 30 | VehicleRacePacketDestroyProxiedVehicleRaceMine |
| 14 | VehicleRacePacketUpdateVehicleRaceProgressData | 31 | VehicleRacePacketUpdateVehicleRaceMineData |
| 15 | VehicleRacePacketCreateProxiedVehicleRacePickup | 32 | VehicleRacePacketResetNpcsRequest |
| 16 | VehicleRacePacketDestroyProxiedVehicleRacePickup | 33 | VehicleRacePacketResetProxiedVehicleRace |
| 17 | VehicleRacePacketPickupHitRequest | 34 | VehicleRacePacketSetRaceConfig |

Cross-check against the captured launch order: `34 SetRaceConfig` arrives first (t=1.563 s), then
`11 CreateProxiedVehicleRace` once per racer with the kart body asset (t=1.588 s), then
`5/6 UpdateVehicleRaceClientData/ServerData` and `14 UpdateVehicleRaceProgressData` carry the bulk
streams, `7/8 GateHit/VehicleHit`, `9 RegisterVehicleRacePlayer`, and `15-23` the pickup/boost
traffic. Subtypes 1-4 and 10 were not observed in the capture window.

## Unknown fields (explicit)

* Table 2 and table 3 semantics are unrecovered: rows are `key + 3 i32`, and no consumer or name is
  recovered. Their captured samples start `(1,(1,1,1)), (1,(1,2,2)), (1,(1,4,4))…` and
  `(1,(1,1,1)), (1,(1,2,3)), (1,(1,16,2))…`.
* Table 4's byte string is consumed by `FUN_00894b10`; the consumer is not named, and the only
  observed values are single digits.
* Games-row field 4 (37 on row 1; 65/68/60 on rows 4-6) has no recovered meaning. The pinned
  community writer names the same position `Unknown` and leaves it at 37 in its test row.
* The captured catalogue content is test data; the retail per-game rows and names beyond 3052/3053
  are not recovered here.

## Synthetic vectors for the future builder

Private, derived vectors (no captured bytes) live in `vectors.json` with three encodings each: body,
inner packet, and the tunnelled form with outer 5 / flag 1:

| Vector | Body | Inner | Tunnelled |
| --- | --- | --- | --- |
| `lobby-minimal-kart` | 40 B | 48 B | 55 B |
| `lobby-one-each` | 113 B | 121 B | 128 B |

`lobby-minimal-kart` body hex (games table = one Kart Race row, all four option tables empty):

```
01000000 01000000 01000000 01000000 25000000 ec0b0000
00000000 00000000 00000000 00000000
```

The reproducer encodes each vector, decodes it with the recovered layout, and asserts field equality
plus exact consumption, so the vectors cannot silently drift from the framing. A future Rust
serializer should emit the inner packet form; the gateway's tunnel wrapper supplies outer/flag/length.

## Verification

```
& '<lab>/tools/wiki-env/Scripts/python.exe' trace-kart-lobby-framing.py
{"body_sha256": "51c8f011c781fe376496c851889380ba36f6bbed911daa60eb964ecc71fd5a0c",
 "consumed": 2378, "subtypes_mapped": 34,
 "proof_sha256": "da47c4165e6273dce2f3785ab89e8b2589618df7d1aa471deb0820b5730b5b16",
 "vectors_sha256": "edfd1fcad7195c83e2e3ec12adb53817b20043f664ff606ccf3ba1eb079025ef"}
```

The reproducer asserts the native parse chain and each table's field count directly from the Ghidra
export, decodes both captured 102/2 bodies and requires exact consumption, extracts all 34 subtype
constants and their class vftables, encodes and re-decodes the synthetic vectors, then writes
`proof.json` and `vectors.json` with `write_exact` semantics (re-running produces identical files).

Cross-links: [[Research/Kart Racing Restoration]], [[Research/Go Kart Native Integration Readiness]].
