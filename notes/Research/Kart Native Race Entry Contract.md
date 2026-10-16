# Kart Native Race Entry Contract

Read-only research, 2026-10-09 (UTC). Scope: recover the concrete native
kart race-entry contract - the client request that starts a race, the ordered
server start (scene + config + vehicle) and the counter replies - from the
pinned original capture, its preserved frames, the client decompile and the
community name tables; state the next launch binding and the real absences. No
gateway/content/source/save/UI/release/process change; no client launched; no
finish/reward spoofing.

Owned artifacts:

| Path | Role |
| --- | --- |
| this note | contract, anchors, absences |
| `lab/reports/kart-race-entry-contract-20261009T1735Z/probe-race-entry-contract.py` | reproducer over the pinned index, frames and decompile |
| `lab/reports/kart-race-entry-contract-20261009T1735Z/proof.json` | write_exact proof (subtype table, event order, field maps, writes) |
| `lab/reports/kart-race-entry-contract-20261009T1735Z/run.log` | recorded run, exit 0 |

Inputs (sha256 in `proof.json`): `racing-application-index.json`,
`racing-*.bin` frames under `lab/reports/original-minigame-packets`,
`decompiled/client/FreeRealms/pseudocode.c` (`f84a5867...`),
`ofrserver/Server/LibSOE/Core/PacketCodes.cs`,
`Sanctuary/src/Sanctuary.Packet.Common/Extensions/PacketReaderExtensions.cs`.

## Verdict first

The entry contract is a **client-driven `33/9` register exchange inside a
scene the server already selected**:

```
(server) 41/114 details + op31 scene "sg_kt_track" + spawn
client  102/1 -> server 102/2 catalogue (row 0 = Kart Race)
(server) 41/114 state 4 + 41/2 PlayerEnter  (encounter 917566 / instance 1008)
client  33/9  RegisterVehicleRacePlayer    (session u64 + 0x2a6ac44a + 1)
server  33/34 SetRaceConfig
server  33/11 CreateProxiedVehicleRace      (body asset + 400.0 + attribute block) x6
server  33/5  UpdateVehicleRaceClientData   (96 B)  + 33/14 ProgressData (84 B)
client  33/2  UpdateVehicleGamePlayerState  (session u64 + 2)
server  33/3  player count = 1, 33/1 game state = 3
server  33/4  npc count = 7 (+ later 33/1 = 4), then the racer streams
```

The client never names the track: `33/9` carries only the session id and two
u32s (`0x2a6ac44a`, `1`; semantics unrecovered). Race identity is server-side
content: encounter `917566` / instance `1008`, scene `sg_kt_track`, spawn
`719.61, 72.0, 421.5`, and the kart body asset inside `33/11`.

## Ordered capture (times relative to the 33/9 register)

| t (s) | Frame | Dir | Packet | Decompiled class | Bytes | Role |
| --- | --- | --- | --- | --- | --- | --- |
| -20.224 | 135 | S->C | 41/114 | `EncounterDetailsResponsePacket` | 418 | details, state 2, spawn, session |
| -15.942 | 723 | S->C | op31 | scene load (name string) | 87 | scene `sg_kt_track`, same spawn, `sky.xml` |
| -1.034 | 15232 | C->S | 102/1 | `LobbyGameDefinitionPacketDefinitionsRequest` | 11 | empty definitions request |
| -0.824 | 15239 | S->C | 102/2 | `LobbyGameDefinitionPacketDefinitionsResponse` | 2393 | 6 rows, row0 = Kart Race |
| -0.069 | 15449 | S->C | 41/114 | `EncounterDetailsResponsePacket` | 398 | state 4, same encounter/spawn |
| -0.069 | 15449 | S->C | 41/2 | `EncounterPacketPlayerEnter` | 28 | encounter/instance, zeros, flag 0 |
| 0.000 | 15480 | C->S | 33/9 | `VehicleRacePacketRegisterVehicleRacePlayer` | 27 | **the client request** |
| +0.529 | 15933 | S->C | 33/34 | `VehicleRacePacketSetRaceConfig` | 95 | race config block |
| +0.554 | 15934 | S->C | 33/11 | `VehicleRacePacketCreateProxiedVehicleRace` | 6177 | own kart + attribute block |
| +0.554 | 15948 | S->C | 33/5 | `VehicleRacePacketUpdateVehicleRaceClientData` | 96 | bulk client data |
| +0.554 | 15948 | S->C | 33/14 | `VehicleRacePacketUpdateVehicleRaceProgressData` | 84 | session + FLT_MAX sentinel |
| +0.627 | 15950 | C->S | 33/2 | `VehicleRacePacketUpdateVehicleGamePlayerState` | 23 | join |
| +0.995 | 16298 | S->C | 33/3 | `VehicleRacePacketUpdatePlayerCount` | 15 | 1 |
| +0.995 | 16298 | S->C | 33/1 | `VehicleRacePacketUpdateVehicleGameState` | 15 | 3 |
| +3.026 | 17362 | S->C | 33/5/14/1/4 | same classes | 96/84/15/15 | state 4 + `npc count` 7 |
| +74.203 | 42954 | S->C | op31 | scene return | 81 | `FabledRealms` |

Between the catalogue and the register the server also sends the encounter
entry bundle (41/114 + 41/2, frames 15449/15450) and world/entity traffic
(op `150` 82119 B, `147`, `35`, `38`, `39`, `129`); the register follows
~0.1 s after the player-enter. A second `102/1` + `102/2` pair follows the race
at +82.4 s and its response is **byte-identical** to the first (proof
`second_catalogue_identical`).

## Recovered field maps

All frames: `u16 outer 5 | u8 flag | u32 remaining (=2+body) | u16 inner op |
body`. For op 33/41/102 the body starts with `u16 subtype`; for op 31 the body
starts with `u32 name length`.

* `102/1` - 11 bytes, empty request.
* `102/2` - `i32 body length 2378`, then **five tables** in the registered
  production codec's wire order (`src/kart_lobby_native.rs`; native readers
  `FUN_008d67a0` + `FUN_008bfc00`): counts `[6, 30, 43, 37, 17]` for
  games / options / values / display / settings. games = `u32 key + i32 id +
  i32 game_type + i32 unknown + i32 name_id`, row0 = `1,1,1,37,3030`
  (**Kart Race**; community comment `// 1 - Kart Race`), rows 1-5 = `2..6`
  with unknowns `0/0/65/68/60` and name ids `0/0/0/0/417078`; options and
  values = `u32 key + [i32; 3]`; display = `u32 key + i32 unknown + i32
  locale id + i32 byte length + bytes`; settings = `u32 key + [i32; 5]`.
  The 2378-byte body is consumed exactly - no trailing bytes, body sha256
  `51c8f011`. The *meanings* of the options/values triples, the games 4th
  word, the display 2nd word and the settings words stay unnamed.
  Sanctuary's writer produces only the single row0 shape with `Unknown 37`.
* `33/9` (C->S, 16-byte payload) - `u64 session 91ca73956acc524b`,
  `u32 0x2a6ac44a`, `u32 1`. The u64 is the same session seen in 41/114,
  33/2 and 33/14.
* `33/34` (S->C, 84-byte payload) - leading `u32 3`, `u32 1`, `f32 1.0`,
  `f32 1.0`, zeros, `f32 1.0`, zeros, `f32 -1.0`, zeros, then
  `f32 716.0, 82.0, 423.0`, `f32 1.0` and two more dwords. The trailing
  triple is **not** the scene spawn (`719.61, 72.0, 421.5`); field semantics
  stay unrecovered.
* `33/11` (S->C, payload 6166 first) - `u32 name length 22`,
  `kt_kart08_body_bbe.cdt`, `f32 400.0`, then the attribute block. All six
  captured sub-11 frames carry `f32 400.0`; their bodies are
  `kt_kart08_body_bbe.cdt` (own kart) and
  `sg_kt_speedster_body_bbec_002.cdt` / `sg_kt_zipster_body_bbec_002.cdt`
  (proxied racers). The attribute block is a run of small length-prefixed
  strings: `bluestripe`, `trophy`, `frlogo`, the `.adr` part assets,
  `Kart08Audio`, and vehicle sound events (`Vehicle_Boost_Fired`,
  `Vehicle_Crash_Small/Medium/Large`).
* `33/5` (S->C, 85-byte payload) and `33/14` (S->C, 73-byte payload) - bulk
  racer/progress data. `33/14` starts with the session u64 and contains
  `0x7f7fffff` (FLT_MAX); layouts are only size/sentinel-mapped, so neither is
  authoritative position.
* `33/2` (C->S, 12-byte payload) - session u64 + `u32 2`.
* `33/1` / `33/3` / `33/4` (S->C, 4-byte payload) - `u32 3` (game state),
  `u32 1` (player count), `u32 7` (npc count); later frames repeat with 4.
* `op31` scene load (78-byte body after the inner op) decodes in the registered
  production codec's field order (`src/zone_transition.rs`, opcode 31
  `PacketClientBeginZoning`): `u32 11`, `sg_kt_track`, `i32 type 2`,
  position `4 x f32` = `719.609985, 72.0, 421.5, 1.0` (first three
  byte-identical to the 41/114 spawn), rotation `4 x f32` = `0, 0, 1, 0`,
  `sky.xml`, tail (tutorial 0, unknown 1, id 0, radius 0.0, geometry id 101,
  bools 1/0). The return frame carries `FabledRealms`.
* `41/114` - `u32 encounter 917566`, `u32 instance 1008`, repeat 917566,
  state 2 (frame 135) / 4 (frame 15449), `u32 0xde2106fb`, session u64,
  pair, spawn at split-body offset 49 (payload offset 47; located by exact
  bytes, the flag/bool boundaries before it stay unmapped). `41/2` - encounter, instance,
  8 zero bytes, flag 0 on first entry.

## Subtype table (inner opcode 33, 34 classes)

Extracted from the preserved Ghidra export (ctor + vftable) and cross-checked
identical to both community maps (OFR `BaseVehicleRacePackets` enum, Sanctuary
`33 =>` name table): 1 `UpdateVehicleGameState`, 2
`UpdateVehicleGamePlayerState`, 3 `UpdatePlayerCount`, 4 `UpdateNpcCount`,
5 `UpdateVehicleRaceClientData`, 6 `UpdateVehicleRaceServerData`,
7 `GateHit`, 8 `VehicleHit`, 9 `RegisterVehicleRacePlayer`,
10 `VehicleRaceCountdownStatus`, 11 `CreateProxiedVehicleRace`,
12 `DestroyProxiedVehicleRace`, 13 `KickProxiedVehicleRace`,
14 `UpdateVehicleRaceProgressData`, 15 `CreateProxiedVehicleRacePickup`,
16 `DestroyProxiedVehicleRacePickup`, 17 `PickupHitRequest`,
18 `PickupHitResponse`, 19 `UseItemRequest`, 20 `UseItemResponse`,
21 `ItemExpired`, 22 `BoostAwardRequest`, 23 `BoostAwardResponse`,
24 `CreateProxiedVehicleRaceMissile`, 25 `DestroyProxiedVehicleRaceMissile`,
26 `UpdateVehicleRaceMissileData`, 27 `CreateProxiedVehicleRaceExplosion`,
28 `DestroyProxiedVehicleRaceExplosion`, 29 `CreateProxiedVehicleRaceMine`,
30 `DestroyProxiedVehicleRaceMine`, 31 `UpdateVehicleRaceMineData`,
32 `ResetNpcsRequest`, 33 `ResetProxiedVehicleRace`, 34 `SetRaceConfig`.
Ctor addresses/line numbers are in `proof.json`.

## Independent reproduction

```
& '<lab>/tools/universal-modder-env/Scripts/python.exe' \
  lab/reports/kart-race-entry-contract-20261009T1735Z/probe-race-entry-contract.py
```

The probe re-derives the subtype table from `pseudocode.c` and asserts it
equals both community maps; joins the preserved index to the frame files by
sha256 (the file-name number is a capture counter, not the index `sequence`);
asserts the launch order and relative times; parses the field maps above;
asserts every captured sub-11 frame decodes to a body asset plus `400.0`;
and runs the writer, which reproduces the `33/9` request, the `33/2` join, the
`33/1/3/4` counters and the `op31` scene frame **byte-identically**, and
reproduces the `33/11` header prefix (`name + 400.0`) with the attribute block
explicitly preserved from the captured template.

## Absences and limits

* No community serializer exists for the 33 family: `PacketCodes.cs` has the
  enum only, Sanctuary has the name table only (bounded search).
* `BaseLobbyPackets` (Join/Create/Start) has a name enum but no opcode value in
  the pinned community sources, and no such inner opcode appears in the
  captured launch window, so it is not part of this contract.
* Subtypes 12, 13, 19, 20, 21, 24-33 are named but **not captured** in this
  capture; their payload layouts are intended-but-absent.
* Wire layouts that are recovered and complete: the op31 frame - including
  its 16-byte tail (tutorial, unknown, id, update radius, geometry id, two
  bools) - is fully decoded by the native reader `FUN_008e4ae0`, the
  unchanged community writer and the registered `src/zone_transition.rs`;
  the `102/2` body is fully decoded by the registered
  `src/kart_lobby_native.rs` (five tables, exact consumption). What stays
  open here is field *meaning* only: the two `33/9` u32s, most `33/34`
  words, the `33/11` attribute semantics, the `41/114` flag/bool
  boundaries, and the unnamed `unknown`/`id`/triple words inside the
  otherwise complete 102/2 and op31 layouts.
* No finish/results message is recovered: the 33 stream simply stops; the only
  follow-up is a `141` matchmaking pair and a fresh `102/2`. The
  `0xde2106fb` constant is shared across racing/derby/soccer and is not proven
  to be a scene selector (see the correction in
  [[Research/Kart Scene Load Binding]]); the scene is carried by op31.

## Next launch binding (what the lead must wire)

1. On the client's `33/9` (after 102/1 -> 102/2 and the encounter
   player-enter), reply with `33/34`, one `33/11` per vehicle, `33/5` and
   `33/14`; answer `33/2` with `33/3` + `33/1` and later `33/4`. Byte
   templates and exact frames are pinned in `proof.json`; only the session id,
   spawn, sky and body assets are caller-supplied content.
2. Send the scene with op31 when the player enters the speedway, and the
   encounter with 41/114 + 41/2 using the same session id (the capture does
   this ~16-20 s before the register). Production already carries the reviewed
   op31 codec (`src/zone_transition.rs`), the 102 lobby codec
   (`src/kart_lobby_native.rs`) and the 33/14 progress codec
   (`src/kart_progress_native.rs`); what is missing is 33/9 handling, the
   33/34 + 33/11 + 33/5 start bundle, the 33/1/3/4 counters and this ordering.
3. Keep finish/reward out of the binding: no captured result message exists,
   so a race cannot be completed or paid from this contract alone.

## Correction (v2, 2026-10-09)

The first version of this note (sha256 `0eabed9f…`) called the `102/2`
trailing catalogue block and the op31 16-byte tail "unrecovered". Both were
wrong for the current wire layouts: the `102/2` body decodes completely as
five tables in the registered `src/kart_lobby_native.rs` (counts
  `[6,30,43,37,17]`, exact consumption, body sha256 `51c8f011…`), and the op31
  frame decodes completely per the native reader `FUN_008e4ae0`, the unchanged
  community writer and the registered `src/zone_transition.rs`. Only field
  *semantics* remain unnamed. The corrected decode is in
  `probe-race-entry-contract-v2.py` /
`proof-v2.json`; the v1 `proof.json`, its script/run and this note's v1 hash
stay preserved in `receipt.json` + `final-hashes.txt`.

Cross-links: [[Research/Go Kart Native Integration Readiness]],
[[Research/Kart Community World Binding]],
[[Research/Kart Speedway Entry Binding]],
[[Research/Kart Native Lobby Codec]], [[Research/Kart Native Progress Codec]].
