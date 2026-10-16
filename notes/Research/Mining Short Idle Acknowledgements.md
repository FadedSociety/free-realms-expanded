# Mining Short Idle Acknowledgements

Fix for a protocol bug in the shared TYPE 8 Mining engine: `OnIdleMsg` reused the
chain coordinate parser, which required three `x,y` pairs, so the native client's
normal one- and two-tile idle acknowledgements were refused.

## Recovered wiring

| Step | Source |
| --- | --- |
| The client accumulates fired tiles in `sNowIdle` and sends the string as soon as it is non-empty | `core/Grid.as` `UpdateIdle` 1356-1363 |
| `SendIdle` wraps it as `OnIdleMsg` | `GameClientNetwork.as` 26-29 |
| The server splits on `,`, pops the one trailing empty field and forwards **every** remaining pair | `GameLevel.as` `OnIdleMsg` 543-552 |
| `NowIdle` loops `length / 2` entries and marks the idle-pending / appearing tiles idle; there is **no minimum** | `core/ServerGrid.as` `NowIdle` 78-102 |

Nothing in that path imposes a three-tile floor. The three-tile minimum belongs
to the chain reader alone: `GameData.nMINIMUM_CHAIN_LENGTH = 3`
(`GameData.as` 47) and `ServerGrid.ValidateAndRemoveChain` 103-133 rejects a
shorter list before it is even walked.

## Change

`src/mining.rs` now has one shared list parser, `coordinate_list(list,
minimum_values)`, with the recovered trailing-comma contract intact (one
trailing empty field is original data, any other empty field, an odd field
count, a non-numeric field or a coordinate above 4096 is refused):

- `coordinates` (chains) keeps the six-value floor, so the three-tile minimum
  and every chain rule are unchanged.
- `idle_points` (new) uses a two-value floor, so a single `"4,4,"` is a complete
  idle acknowledgement while `"4,4"` with an empty interior field is still
  malformed.
- `powerup_points` keeps its existing two-value floor on the shared parser; no
  powerup behaviour changed.

The `OnIdleMsg` handler body is otherwise untouched: it parses first, then only
applies marks while the run is in `Playing`, so terminal and pre-start runs
still accept a well-formed acknowledgement without changing state, paused play
still applies it, and any refusal leaves the whole run unchanged because the
handler works on the per-message state clone.

## Tests

`prototype/osfr-tools/tests/mining_idle.rs` (new, production crate imports,
synthetic states built from the production `write_content`):

- RED then GREEN: one-tile `"4,4,"` and two-tile `"4,4,5,4,"` idle
  acknowledgements are accepted and mark exactly those cells idle.
- The chain path still refuses one- and two-tile lists and still accepts the
  legal three-tile chain.
- An idle list with an out-of-board coordinate is refused atomically, including
  the case where an earlier pair was valid.
- Odd field counts, empty interior fields, non-numeric fields, oversized
  coordinates and an empty list are all refused with the run unchanged.
- Terminal runs accept a well-formed acknowledgement without change and still
  refuse malformed input; paused play still applies the acknowledgement.

The RED run is preserved as `lab/reports/mining-idle-ack-20261009T2000Z/red.log`
(3 failed / 3 passed) and the GREEN run as `green.log` (6 passed).

Focused suites after the fix: `mining_idle` 6, `mining` 21, `mining_dead_board`
3, `mining_move_detection` 3 (+1 ignored private snapshot), `mining_review` 13,
`mining_shape_parity` 4, `mining_tile_rendering` 3, `mining_powerups` 22,
`type8_gateway` 10, `type8_harvesting_gateway` 11 — all green; `check --lib`
clean. No content, save, UI, Cargo or shared-file change, and no release build.

Pins (before this note's own edit): `src/mining.rs`
`8b32d7c5cf646f794d73b4f4426ecd053f5273f3cf63a419429e5512d625c83d` and
`tests/mining_idle.rs`
`8c2a45f4bc725e9bc20193a7b83ac43f03f7eb351b96934d3c64f2a2745d82f3`. The live
values, together with the RED and GREEN logs, are in
`lab/reports/mining-idle-ack-20261009T2000Z/receipt.json`.
