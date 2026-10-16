# Harvesting Border Fix

Status: **implemented; GREEN captured for the border target and the current focused Harvest family
(53 passed)** (2026-10-09 UTC). Receipt:
`lab/reports/harvesting-border-fix-20261009-1417Z/`. Evidence label:
**RECONSTRUCTED padded geometry over the research in `notes/Research/Harvesting Border
Compatibility.md`**. No native claim, no content package edit.

Owned paths for this fix:

| Path | Role |
| --- | --- |
| `prototype/osfr-tools/src/harvesting.rs` | padded shape constants/helper, `board_spec`, bounds-aware `validate_content`, notice/doc text |
| `prototype/osfr-tools/tests/harvesting_border.rs` | 3 vectors (RED first) |
| `lab/reports/harvesting-border-fix-20261009-1417Z/` | RED log, blocked-GREEN log, receipt |

## Change

- `RECONSTRUCTED_BOARD_WIDTH/HEIGHT` 10 -> 11 and a new `RECONSTRUCTED_BORDER = 1`: row 0 and column 0
  stay empty because the original `checkFalls`/`addColumnFall` loops never visit index 0
  (`ServerGrid.as`/`Grid.as`, cited in `Harvesting Border Compatibility.md`).
- New `padded_engine_shape(width, height, border, playable_rows)`: `0` borders, the same 90 regular
  cells at x1..10 / y1..9, and the drop row at y10 over x1..10. `engine_rectangle_shape` is kept
  unchanged for its existing reconstruction tests.
- `validate_content` now derives the playable rectangle exactly like the shared engine's
  `board_bounds` (non-empty, non-dropzone cells; rectangle must be full; drop row directly below;
  no dropzone outside) instead of assuming the playable area starts at index 0. A leading empty
  border is accepted; a hollow border violation is still refused.
- `RULE_NOTICE` and the module doc now say "11x11 padded grid" instead of "10x10 grid"; the
  screenshot/fidelity-gap wording is unchanged.
- Unchanged: `selectable`, `collectable`, `tiles` (8 kinds), `goals`, `outputs`,
  `duration_seconds`, `seed`, all spawn percents, `meter_id`, profile 45 and the farming assets.

## RED

`cargo.ps1 test --test harvesting_border` on the pre-fix source (`red-tests.log`):

```
test padded_geometry_preserves_parameters_and_playable_cell_count ... ok
test emitted_shape_pads_the_original_zero_edges ... FAILED       left: (10, 10) right: (11, 11)
test padded_content_starts_on_the_real_engine_with_inert_edges ... FAILED   left: (10, 10) right: (11, 11)
```

## GREEN

`cargo.ps1 test --test harvesting_border` on the padded source (`green-tests.log`, 2026-10-09 07:24
local):

```
test emitted_shape_pads_the_original_zero_edges ... ok
test padded_geometry_preserves_parameters_and_playable_cell_count ... ok
test padded_content_starts_on_the_real_engine_with_inert_edges ... ok
test result: ok. 3 passed; 0 failed; 0 ignored
```

The first attempts after the relink were refused while the fresh binary sat inside the Smart App
Control scan window; the verbatim refusals are preserved unqualified as
`geometry-family-sac-block-last.log` (and the captured copied-exe attempt as
`policy-block-copied-exe-attempt.log` where present):

```
could not execute process `...\deps\harvesting_border-473efdd07bf91251.exe` (never executed)
Caused by: An Application Control policy has blocked this file. (os error 4551)
```

The policy was not weakened, disabled or bypassed and no binary was renamed or altered; the retry
with the plain project `scripts/cargo.ps1` succeeded once the scanner released the file. An isolated
`CARGO_TARGET_DIR=%TEMP%\osfr-target-harvest` build was refused the same way at its first build
script; that attempt is recorded historically and was not repeated.

Current focused Harvest family (`family.log`, runtime-border receipt, 2026-10-09 07:3x local):
harvesting 17, harvesting_job_binding 3, harvesting_review 4, harvesting_reward_catalogue 4,
harvesting_runtime_border 2, harvesting_runtime_tiles 3, harvesting_tile_registration 9,
type8_harvesting_gateway 11 -> **53 passed, 0 failed**.

## Assertion updates applied (exact, authorized)

1. `tests/harvesting.rs::observed_diamond_shape_and_engine_rectangle_stay_separate`: the emitted
   comparison now uses `padded_engine_shape(11, 11, 1, 9)` (121 cells: 90 `1`s, 10 `4`s, row 0 and
   column 0 zero); `engine_rectangle_shape(10, 9)` is kept and still asserted as the mining-style
   rectangle.
2. `tests/harvesting.rs::content_guards_reject_unknown_or_mutated_configuration`: `shape[90] = 1`
   (a no-op inside the padded playable area) became `shape[0] = 1`, the index-0 border cell the new
   gate refuses. The "last ten cells are dropzone" assertion still holds unchanged.
3. `tests/harvesting_review.rs::emitted_geometry_is_engine_compatible_and_labels_the_fidelity_gap`:
   width/height `10` -> `11`; the drop-row check now reads `shape[(height - 1) * width + 1..]`; row 0
   and column 0 are asserted `0`; the playable x1..10 / y1..9 rectangle is asserted `1`. Quotas,
   rewards and every other assertion are untouched.
4. `tests/type8_harvesting_gateway.rs::spec()` (authorized fixture edit): `tiles` now carries the
   four regular kinds `(401..404, category 400, icons 1..4)` followed by the four collectables;
   required by the runtime tile gate, no assertion weakened.

`tests/harvesting_review.rs::harvested_bundles_grant_once_from_a_serialized_completion_state` needed
no change: the engine's `validate_run` does not bounds-check stored board positions and the vector
passed unchanged.

Cross-links: [[Harvesting Border Compatibility]], [[Harvesting Runtime Tile Guard]],
[[Harvesting Tile Registration]], [[../MMO Work Queue]].
