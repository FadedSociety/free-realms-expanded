# Mining Drop and Reshuffle Parity Review

Read-only independent review of the two mining parity repairs. No source,
content, save, UI, launcher or process was changed. Owned artifacts: this note
and `../lab/reports/mining-parity-review-20261009T134435Z/`.

Verdict: both repairs are bounded, match the recovered original movie behavior,
and the second does not disturb the first. The pre-repair build reproduces the
captured drift; the current build does not. Native acceptance is still the
lead's gate and is not claimed here.

## Frozen revisions (re-hashed)

| revision | sha256 | bytes |
| --- | --- | --- |
| base `e6e4` | `e6e421d8e7cd0e923b9f033638f24fb04d648613e9b6719e78a28c3fc45e090e` | 58636 |
| mid `ec611` | `ec611ec094fb86054651a14eafde15244fc3e19657dc7d0928514ee695b0adec` | 58183 |
| current `6c59` | `6c59ae0ce97a998c062e5cc25658c0c4c8accd150e8db0ac7550c756c1ed38ed` | 58345 |

`mining.rs` was the only source file either repair touched; the guard files are
owned elsewhere and were read only: `gameplay.rs`
`996d33865966fcf8b7d0cc5f0e761f2afdeff9230cb64758e0f303a3ac21d8fa`,
`inventory.rs` `dccbf8006bd4d81950410825406b927d7dd1c3f55b89445a4d680ff7395d51a7`.

## Exact bounded diff

Re-derived with `git diff --no-index --unified=3`; the hunk bodies equal the
builder's stored patches, whose hashes match their receipts
(`3e8adc1482f4d215…` reshuffle, `47138762e48e4727…` shape). Details:
`rederived-e6e4-to-ec611.diff`, `rederived-ec611-to-6c59.diff`,
`diff-verification.json`.

- Repair one (`e6e4` -> `ec611`, +2 / -12, three hunks): the chain-update path
  and the collect-update path both lose `&& !has_bottom_collectable(...)`, and
  the helper itself (7 lines plus its blank separator) is deleted. Nothing else
  changes; `has_bottom_collectable` appears zero times in the current file.
- Repair two (`ec611` -> `6c59`, +3 / -1, one hunk): shape `1` now calls
  `random_tile(run, spec, false)` with a two-line anchor comment;
  shape `2` and the queue/refill draws keep `true`. No constant, quota, seed,
  duration, scoring or message line moves.

## Original parity evidence

- `core/ServerGrid.as:751-753` — `if(!this.HasMove()) { this.LevelReshuffle(); }`
  with no bottom-collectable suppressor. `HasMove` (`:189-221`) returns true for
  a powerup press-me or for a wildcard/selectable cell with more than one
  matching neighbour; collectables are not moves. `LevelReshuffle` (`:321-333`)
  rebuilds map and queues and sends `SendLevelReshuffle(RESHUFFLE_NOMOVES)`.
- `core/ServerGrid.as:357-361` — map shape `1` = `TILE_BUILDREGULAR`, `2` =
  `TILE_BUILDANY`.
- `core/ServerTileGenerator.as:99-110` — `oTypeRandomizer` includes
  `TILE_COLLECTABLE`; `oRegularRandomizer` is selectable/hazard/wildcard only.
  `generateTile` (`:112-137`) maps `BUILDREGULAR` to the regular pool and
  `BUILDANY` to the type pool, so shape `1` never draws a collectable while
  shape `2` and the queue/refill draws still can.

## Independent three-way probe

Private crate `../lab/reports/mining-parity-review-20261009T134435Z/probe`
compiles all three revisions from their hash-verified files and drives them
through the real `Type8Engine` frame path; `probe-green.log` = 2 passed / 0
failed, full matrix in `matrix.log`.

| vector | base `e6e4` | mid `ec611` | current `6c59` |
| --- | --- | --- | --- |
| captured chainless + bottom ore, chain path | reshuffles 0, queue 57 | 1, queue 70, notice+build map | identical to mid |
| captured chainless + bottom ore, collect path | reshuffles 0, queue 59 | 1, queue 70; collected `[1,0]`, score 3500 | identical to mid |
| independent checkerboard chainless, chain path | 0 | 1 | 1 |
| refill restores a legal 401 run | 0 | 0 | 0 |
| persisted run loaded, before any frame | board untouched | untouched | untouched |
| shape 1 at forced 100% collectable, seeds 18-21 | 64/64 cells ore | 64/64 ore | 0/64 ore |
| shape 2 at forced 100% | collectables present | present | present |
| shape 1 at wildcard 100% + collectable 100% | 64/64 wildcard | wildcard | wildcard |
| short refill queue through the chain path | queue 66, 20 ore | byte-identical | byte-identical |

Conservation: chain path keeps chains 7, collected `[0,0]`, score 3400 and
remaining 178329 ms in all three revisions; the collect path keeps
`OnQuotaUpdateMsg`, collected `[1,0]`, score 3500 and remaining 172329 ms in all
three; the rebuild only adds the queue refill to 70 and increments
`reshuffles`. `mining_spec()` serializes byte-identically across the three
revisions (seed 18, duration 180 s, quotas 15/12), so no quota was lowered, no
seed replaced and no payout was faked.

## Limits

- This review executed the Rust engine, not the native client. The captured run
  `125556` remains the drift-of-record; native acceptance is the lead's gate.
- The two builder test files were re-hashed but not modified:
  `tests/mining_dead_board.rs` `f2d71f4eb36d9acf…`,
  `tests/mining_shape_parity.rs` `90c9b8eb85ab6969…`.
