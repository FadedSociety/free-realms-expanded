# Mining Wildcard Move Detection

Status: implemented; RED and GREEN captured, 2026-10-09 UTC. Receipt:
`lab/reports/mining-move-detection-20261009-1450Z/`. Evidence label: **RECONSTRUCTED move check over
ORIGINAL chain rules**. No release build, no UI/process/save edit, no other handler touched.

Owned paths for this repair:

| Path | Role |
| --- | --- |
| `prototype/osfr-tools/src/mining.rs` | `has_move` replaced (plus the two narrow helpers `neighbour`, `chain_values_match`) |
| `prototype/osfr-tools/tests/mining_move_detection.rs` | 4 vectors (RED first) |
| `lab/reports/mining-move-detection-20261009-1450Z/` | RED/GREEN logs, Mining and Harvest family logs, receipt |

## Defect

Native run 143211 paused with 21 chains, **every one of the 196 tiles idle** and one reshuffle
already applied; the client showed no legal path. Its frozen snapshot
(`lab/runs/client-baseline-20261009T143211Z/character-chain21-no-move.json`, sha256
`e3885ebeaab33c9f15232f13a9e70333b4d8a8ccd24db8077b30d14927b1bd24`) carries wildcard 200 at (3,3),
a goal collectable 602 at (3,4) and a 403 at (4,3).

The old `has_move` seeded a flood fill from the wildcard and treated **any** neighbour as a match
(`other == value || other == TILE_WILDCARD || value == TILE_WILDCARD`), so goal collectables, mixed
colours and powerups all counted toward the three-tile threshold. An independent exhaustive
three-tile finder over the snapshot returns **0** legal paths, yet the engine reported a move and
skipped the reshuffle that the client was waiting for.

## Fix

`has_move` now answers "does a legal three-tile path exist", exactly what `validate_chain` accepts:

- three distinct, orthogonally connected cells, each selectable (`400..=499`) or wildcard (`200`);
- every non-wildcard value equal; wildcards bridge and a pure-wildcard triple is legal;
- **idle flags ignored**, because this is the post-chain future-move check that drives the
  reshuffle, not a statement about tiles currently animating.

Any accepted chain of three or more tiles contains such a path, so enumerating each cell as the
centre with two distinct orthogonal neighbours (six pairs) is exhaustive and replaces the flood
fill with a fixed-size scan. Two narrow helpers were added next to it: `neighbour` (bounds-checked
orthogonal step) and `chain_values_match` (the value rule above). No quota, seed, reward, shape,
RNG, score or timer logic changed.

## Verification

| Command | Result | Log |
| --- | --- | --- |
| RED: `cargo.ps1 test --test mining_move_detection` | 1 passed, 3 failed — the snapshot and both wildcard traps did not reshuffle | `red-tests.log` |
| GREEN ordinary: `cargo.ps1 test --test mining_move_detection` | 3 passed, 0 failed, **1 ignored** — the two wildcard traps reshuffle and the straight/bent/wildcard-bridge/pure-wildcard future paths (with idle=false) keep the board | `ordinary-3.log` |
| GREEN private: `OSFR_MINING_NO_MOVE_SNAPSHOT=<lab path> cargo.ps1 test --test mining_move_detection -- --ignored` | 1 passed, 0 failed — the real chain21 snapshot reshuffles | `private-1.log` |
| `--test mining --test mining_dead_board --test mining_review --test mining_shape_parity --test mining_tile_rendering` | 21 + 3 + 13 + 4 + 3 passed | `mining-family.log` |
| `--lib` and the Harvest family (`harvesting`, `harvesting_review`, `harvesting_runtime_tiles`, `harvesting_runtime_border`, `harvesting_tile_registration`, `harvesting_border`, `type8_harvesting_gateway`) | 24 + 17 + 4 + 3 + 2 + 9 + 3 + 11 passed | `harvest-family.log` |

The snapshot vector also re-checks the frozen sha256, asserts all 196 idle flags and reads the live
instance id (`18000004`) so the recovered fall event is accepted.

## Portability and the private capture

`chain21_snapshot_with_no_legal_path_must_reshuffle` is the only vector that reads a private lab
capture (the frozen chain21 snapshot in `lab/runs/`), so it carries
`#[ignore = "requires private native Mining snapshot"]`. The three synthetic vectors stay in the
default run and cover the false-positive shape and every valid three-tile form without any lab
path. Nothing is silently skipped: the ignored test is visible in the ordinary output, and the root
gate runs it explicitly with `--ignored` and `OSFR_MINING_NO_MOVE_SNAPSHOT` pointing at the capture.
The snapshot bytes are never embedded or copied into the repository or the receipt.

## Hashes (Python sha256)

```
da3b5b567bdcaf054d713297af9bae412a367453c8989dc157030f8946fe7a1d  src/mining.rs (after the fix)
6c59ae0ce97a998c062e5cc25658c0c4c8accd150e8db0ac7550c756c1ed38ed  src/mining.rs (pre-fix, frozen EF85 source hash)
```

The worker did not preserve a pre-edit source copy. The root reconstructed the pre-fix revision from
the recorded `has_move` function plus the unchanged remainder and filed it as
`lab/reports/mining-move-detection-20261009-1450Z/reconstructed-mining-6c59.rs`; its sha256 is exactly
`6c59ae0c…`, but its provenance is an **after-edit reconstruction, not a pre-edit preservation**
(`pre-fix-reconstruction.json`: `match: true`, `current_sha256: da3b5b56…`). Everything else in
`src/mining.rs` is unchanged, and the single edited function is `has_move`.

Cross-links: [[Mining Restoration]], [[Harvesting Runtime Border Gate]], [[../MMO Work Queue]].
