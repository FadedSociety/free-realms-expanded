# Kart race finish protocol (inner 33)

Read-only recovery, October 9, 2026 (UTC). Scope: bind the native finish/results path for the kart
race — server state flags, lap counters, session results and the return to the world — from the
pinned Ghidra export, the shipped UI Lua and the already decoded capture bodies. Reused without
re-derivation: the subtype-to-class map and the 102/2 lobby framing from
[[Research/Kart Lobby Definition Framing]]. No production source, content, save, UI, release or
process change. The capture's stream ending is **not** treated as a finish, and the `141` matchmaking
pair after it is **not** treated as results.

Owned artifacts:

| Path | Role |
| --- | --- |
| this note | finish chain, payload layout, captured values, uncertainties |
| `lab/reports/kart-finish-binding-20261009T154032Z/proof.json` | pinned proof, sha256 `09bf7bc2507196f29d5766aac2c52ab67febb447126d6a7c94b3980467d498dc` |
| `lab/reports/kart-finish-binding-20261009T154032Z/vectors.json` | private synthetic finish vectors, sha256 `16a70fce97c09e2d4f595835469224081cd2d293d3436a356b050f76d5eeae63` |
| `lab/reports/kart-finish-binding-20261009T154032Z/trace-kart-finish-binding.py` | reproducer |
| `lab/reports/kart-finish-binding-20261009T154032Z/trace-run.log` | run output |

## Native finish chain

All three packets are dispatched by `FUN_00ba74d0` (vehicle-race family, outer opcode 33):

| Subtype | Class | Payload | Client binding |
| --- | --- | --- | --- |
| 1 | VehicleRacePacketUpdateVehicleGameState | `i32 state` (4 B) | `processor+0x194 = state` |
| 12 | VehicleRacePacketDestroyProxiedVehicleRace | `u64 guid` (8 B) | racer removal; **not** a finish |
| 14 | VehicleRacePacketUpdateVehicleRaceProgressData | 73 B field block (below) | per-racer laps/times + local lap limit |

`FUN_00b9fc80` reads subtype 14 in this exact order (payload-relative offsets, all bounds-checked):

| Offset | Type | Binding |
| --- | --- | --- |
| 0x00 | u64 | racer guid; `FUN_00b9e0c0(guid)` selects the racer object |
| 0x08 | i32 | racer slot/index (0..7 observed); stored at racer+0x1610 |
| 0x0c | f32 | read, NAN-checked, no recovered consumer |
| 0x10 | f32 | racer+0x1614, stored only while the game state is 9; feeds the sort/flags |
| 0x14 | i32 | **laps completed** -> racer+0x15ec |
| 0x18 | u8 | local-racer flag -> processor+0x1a4 |
| 0x19 | i32 | **lap limit** -> processor+0x1a0 |
| 0x1d | f32 | racer+0x1618 |
| 0x21 | f32 | **best lap** -> racer+0x161c |
| 0x25 | f32 | **total time** -> racer+0x1620 |
| 0x29, 0x31, 0x39, 0x41 | u64 x4 | read, no recovered consumer |

Machine-code confirmation (dispatcher `FUN_00ba74d0`): `00ba7813 mov [ebx+0x15ec], ecx`,
`00ba7820 mov [ebx+0x1610], edx`, `00ba782f/f30f1183…1618`, `00ba7840 …161c`, `00ba7851 …1620`,
`00ba7860/786b …1614` (guarded by `[esi+0x194] == 9`), and the local-racer writes
`00ba77c4 mov [esi+0x1a0], eax` / `00ba77ca mov [esi+0x1a4], cl`.

## Results data source and trigger

`FUN_00ce3330` (`VehicleRaceResultsDataSource`) publishes one 40-byte row per racer into
`BaseClient.VehicleRaceProcessor.RaceResults`:

| Row offset | Source | Lua column |
| --- | --- | --- |
| +0x00/+0x04 | racer+0xe60/+0xe64 | Guid |
| +0x08 | racer name object | Name |
| +0x18 | racer+0x161c | BestLapTime |
| +0x1c | racer+0x1620 | TotalTime |
| +0x20 | `(processor+0x1a0 < racer+0x15ec + 1)` | Finished |

So **Finished is client-derived**: a racer counts as finished once their lap counter reaches the
server-sent lap limit; there is no separate server "finished" byte in this packet. Row order (the
Lua `Position` column) is the racer-list order after the sort predicate `00b9abe0`, which groups by
the +0x1614 float and then compares laps (+0x15ec) and total time (+0x1620).

`FUN_00ba6530` shows the results screen when
`processor+0x68 (lap limit) <= racer+0x15ec (laps)` and the racer phase `+0x1624 != 10`: it hides
`RaceLap/RaceBoostBar/RaceItemSlots/RaceOrder/RaceBackwards`, stops the race timer, calls
`FUN_009797a0("RaceResults:Show")`, sets `+0x1624 = 10` and re-emits the local
`VehicleRacePacketUpdateVehicleGamePlayerState` (subtype 2). Three such blocks exist (win/lose/quit
exit paths in the same update function). `FUN_00ba2700` then calls the data-source update and
`FUN_009797a0("RaceResults:Populate")`.

Shipped Lua confirms the consumer: `RaceResultsCol = { Position, Guid, Name, BestLapTime, TotalTime,
Finished }` (line 48292), `RaceResults.Populate` reads
`BaseClient.VehicleRaceProcessor.RaceResults` (line 37360), and `SetPlayerResults` only lists a
player when `finished == 1` (line 37351). The derby variants (`DerbyResults`, `DerbyCrown`) read the
separate `BaseClient.VehicleDemolitionDerbyProcessor.DerbyResults` source and are not part of the
kart race.

## Captured values

Decoding all 9,358 captured subtype-14 frames with the layout above:

* `laps` takes only `0` and `1`; the lap limit is `3` in every frame; the local flag is always `0`.
* 494 frames carry non-zero best-lap/total times: 489 also carry `laps = 1`, while five still carry `laps = 0`. The other 8,864 frames carry `laps = 0` and zero times. This asynchronous cross-tab does not establish an exact simultaneous lap/time transition. The last frames include `best = total = 57.50 s` for one racer and zero times for two others.
* **No captured frame is at or above the lap limit**, so the capture contains no finish and no
  results screen. The stream simply stops at t≈74.5 s while the race is on lap 2 of 3.
* The 7 captured subtype-1 frames carry the state sequence `3, 4, 5, 6, 7, 8, 9` at t≈0.44-13.95 s;
  the client stores each value at `processor+0x194`, and state `9` is the one that also records the
  +0x1614 flag float in the progress handler.
* Subtype 12 never appears in the capture.

The `141` pairs after the race (t≈75.2/85.0 s and t≈94.9/95.1 s) are `BaseMatchmakingPacket`
traffic; nothing in `FUN_00ba2700`, `FUN_00ce3330` or the Lua results path reads them, so they are
recorded only as post-race traffic.

## Synthetic vectors for the future server writer

Private, derived payloads in `vectors.json` (no captured bytes). Each is the inner-33 subtype
payload only; the family/tunnel header is added by the caller.

| Vector | Subtype | Bytes |
| --- | --- | --- |
| `progress_mid_race` | 14 | 73 |
| `progress_finish` | 14 | 73 |
| `game_state_9` | 1 | 4 |
| `destroy_racer` | 12 | 8 |

`progress_finish` payload hex (guid `0x0000004000000001`, slot 0, laps 3 of limit 3, best lap
58.25 s, total 179.75 s, lap timer 61.5 s):

```
0100000040000000 00000000 00000000 00000000 03000000 00 03000000
00007642 00006942 00c03343 0000000000000000000000000000000000000000000000000000000000000000
```

The reproducer encodes and re-decodes each vector with the recovered layout and asserts the
finish condition (`laps >= limit`), so the vectors cannot drift from the binding.

## Uncertainties (explicit)

* Payload fields `0x0c`, `0x10` and `0x1d` have no recovered name; `0x10` is stored on the racer when
  the game state is 9 and participates in the sort predicate.
* The finish sequence is not observed in any capture, so the exact server frame order at the finish
  (state value, final progress block, countdown/results) is reconstructed from the client's
  requirements, not from captured traffic.
* `Finished` and `Position` are client-derived (lap comparison and sort order); the server does not
  send either field in the recovered packets.
* The four trailing u64s in subtype 14 stay unmapped.

## Verification

```
& '<lab>/tools/wiki-env/Scripts/python.exe' trace-kart-finish-binding.py
{"finished_rows": 0, "lap_limit": [3], "lap_values": [0, 1], "progress_frames": 9358,
 "state_sequence": [3, 4, 5, 6, 7, 8, 9],
 "proof_sha256": "09bf7bc2507196f29d5766aac2c52ab67febb447126d6a7c94b3980467d498dc",
 "vectors_sha256": "16a70fce97c09e2d4f595835469224081cd2d293d3436a356b050f76d5eeae63"}
```

The reproducer asserts the dispatcher/parser chain and the exact field order from the Ghidra export,
the `VehicleRaceResultsDataSource` row writes and the results trigger, the Lua column and populate
contract, then decodes every captured subtype-14 and subtype-1 frame and requires the recorded value
distributions. Outputs use `write_exact` semantics; re-running produces identical files.

Cross-links: [[Research/Kart Lobby Definition Framing]], [[Research/Go Kart Native Integration Readiness]],
[[Research/Kart Racing Restoration]].
