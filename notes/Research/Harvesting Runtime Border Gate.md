# Harvesting Runtime Border Gate

Status: implemented; RED and GREEN captured, 2026-10-09 UTC. Receipt:
`lab/reports/harvesting-runtime-border-20261009-1428Z/`. Evidence label: **RECONSTRUCTED runtime gate
over ORIGINAL fall-loop semantics**. No native claim, no Mining change, no content package edit.

## Why

The piece-030 review found a real counterexample: the pre-border eight-kind export
(`29538eb4…`, 10x10 with playable x0..9 / y0..8) still loaded through `Gameplay::load`. The tile
gate validates kinds only, and the shared engine's bounds happily include index 0, so a persisted
document whose regular tiles sit on shape row 0 or column 0 could never fall under the original
`checkFalls`/`addColumnFall` loops (`while (... > 0)`) yet still reached an offer.

## Change (small, shared helper)

- `harvesting::validate_border(shape, width, height)`: refuses any playable shape code (anything
  that is not `0` empty or `4` dropzone) on **shape row 0 or shape column 0**. It inspects the
  spec's own edge codes — the engine's bounds stay private and no public engine API was added.
- `validate_content` calls it, so writer-side validation and the runtime gate agree.
- `Gameplay::load`, in the optional Harvest branch, calls it before the reward checks and before
  `Some(engine)`; the refusal is an `InvalidData` error naming the border. Mining, geometry,
  quotas, seeds, rewards and the regular-tile category are untouched. No exact 11x11 template is
  required — only the index-0 rule.

## RED / GREEN

| Evidence | Result | Log |
| --- | --- | --- |
| RED (before the gate): unpadded eight-kind export loads | 1 passed, 1 failed — `an index-0 playable cell loaded without refusal` | `red-tests.log` |
| GREEN: unpadded export, row-0 rectangle and column-0 rectangle all refused with the tree byte-identical; padded writer output, Mining-only and no-content legacy all load | 2 passed, 0 failed | `green-tests.log` |

The row-0 and column-0 fixtures are shaped so the *shared engine accepts them* (full rectangle with
a drop row); only the border gate refuses, which is exactly the gap the review exposed.

## Focused family (current)

`family.log`: harvesting 17, harvesting_job_binding 3, harvesting_review 4,
harvesting_reward_catalogue 4, harvesting_runtime_border 2, harvesting_runtime_tiles 3,
harvesting_tile_registration 9, type8_harvesting_gateway 11 — **53 passed, 0 failed**.

## Final source hashes (Python sha256)

```
a977c46678aaf55e1314a4f41d8b11038abf69bc6e91a41fb26fe800917d66d1  src/harvesting.rs
d65f89a87cccb12229aa536ab4e684f1b7cfe2f490f1e3106bffef5fab56fa82  src/gameplay.rs
3ce83c9fe1ae91725b0d54db6e8f78e77d2eb844a89d51186e6f3de9f13906cc  tests/harvesting_runtime_border.rs
6c59ae0ce97a998c062e5cc25658c0c4c8accd150e8db0ac7550c756c1ed38ed  src/mining.rs (frozen, untouched)
```
