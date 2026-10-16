# Mining Move Detection Repair Review (read-only)

Repair under review: `has_move` wildcard false-positive fix,
`6c59ae0c…` (reconstructed pre-edit) → `da3b5b56…` (final). Builder receipt
`lab/reports/mining-move-detection-20261009-1450Z/`. No source, content, save,
UI, process or release edit by this review; writes are this note and
`lab/reports/mining-move-detection-review-20261009T145721Z/`.

## Verdict

Repair verified. The old check counted any neighbouring wildcard match as a
move; the final check answers exactly "is there a legal three-tile path", and an
independent differential probe over the captured boards plus 400 random boards
shows the false positive is gone with no over-correction. Native win is not
claimed (the 143211 client is closed; no run in this review).

## 1. Scope of the change

- Pre-edit source recorded as `6c59ae0ce97a998c062e5cc25658c0c4c8accd150e8db0ac7550c756c1ed38ed`;
  root's reconstruction `6c59ae0ce97a998c062e5cc25658c0c4c8accd150e8db0ac7550c756c1ed38ed  bytes=58345  lab\reports\mining-move-detection-20261009-1450Z\reconstructed-mining-6c59.rs` hash-matches it and is labelled
  **reconstructed after the edit**, not a preserved copy.
- Differential `git diff --no-index` reconstructed vs final: **1 hunk, 5,479
  bytes** (`has-move.diff`) — `has_move` replaced by a centre + two distinct
  orthogonal neighbours scan (six neighbour pairs) plus two helpers,
  `neighbour` (bounds-checked step) and `chain_values_match` (selectable
  `400..=499` or wildcard; equal non-wildcard colours; a pure-wildcard triple is
  legal). No other region of the file is in the diff, so RNG, queue, timer,
  score, quota, gravity, collect and dispatch handlers are untouched.
- Final source: `da3b5b567bdcaf054d713297af9bae412a367453c8989dc157030f8946fe7a1d  bytes=59480  project\prototype\osfr-tools\src\mining.rs`.

## 2. Independent differential probe (private crate, no repo edits)

`probe.rs` includes the source at the crate root (private `has_move`/
`validate_spec` reachable) and drives the public `Type8Engine` for the replay.
Because rustc direct binaries were blocked by Application Control, the probe is
built as a private cargo crate under this report (`probe-crate/`, offline
registry) with two binaries: current source and the reconstructed pre-edit file.
The private copies differ from their sources only by converting the 34 leading
`//!` doc lines to `//` (hashes recorded in the manifest); all code bytes are
identical.

| Check | current `da3b5b56` | pre-edit `6c59ae0c` |
| --- | --- | --- |
| captured chain21 (wildcard 200 at col3,row3; ore 602 col3,row4; 403 col4,row3) | `has_move=false`, enum3=false, long=false, old-BFS=true | `has_move=true` (false positive) |
| captured batch2 board | agrees with both enumerators (true) | agrees (true) |
| idle flags ignored | equal with idle all true / all false | equal (true) |
| targeted 10 cases (regular straight/bent, wildcard straight/bent, all-wildcard, mixed colour, wildcard+ore, blocker, corner, empty) | all pass | `wildcard_ore_and_single_selectable` false positive |
| random 400-board equivalence (three fills; 305 positive / 95 negative) | 0 failures | 65 false positives |
| public engine: chainless captured board + fall event | `reshuffles 1→2` (reshuffles) | stays 1 (deadlock) |
| public engine: same board + injected legal 402 triple | stays 1 (suppressed) | stays 1 |

The two independent enumerators (ordered-pair scan and long DFS to 6 cells)
agree with each other and with the final `has_move` on every board tested;
"any legal chain of ≥3 contains a legal 3-tile subpath" held on all 400 random
boards. Long paths also confirm the exact-3 restriction is sound.

## 3. Family verification on the final hash

`cargo test` (repo toolchain, read-only): `mining_move_detection` 3 passed +
1 ignored (the private-snapshot vector is now gated as
`requires private native Mining snapshot`), `mining` 21, `mining_dead_board` 3,
`mining_review` 13, `mining_shape_parity` 4, `mining_tile_rendering` 3, `--lib`
24 — all exit 0 (`family-*.log`, `family-summary.txt`). This matches the
builder's correction (3 synthetic + 1 explicit private root gate). Mining
client 143211 PIDs 14740/32876/32440 are not running; no FreeRealms or gateway
process remains.

## Limits

- The pre-edit file is a hash-matching reconstruction, not a preserved snapshot.
- The probe is engine/mocked only: no native run, no win claim.
- Direct rustc probe binaries were blocked by Application Control; the private
  cargo crate binaries ran and their sources are included here.
