# Harvesting Border Compatibility

Read-only preflight, 2026-10-09 (times UTC). Question: does the original Harvesting client require
non-playable left/top padding, and what is the minimal engine-compatible shape that preserves the
current 10x9 playable geometry? Sources: the extracted Harvesting ActionScript
(`lab/decompiled/minigames/harvesting/scripts/scripts/__Packages/`), the current content package
(`lab/references/opening-harvesting-native-20261009T132512Z/content/harvesting.json`,
`280f5cd7fcca…`), the shared TYPE8 engine (`prototype/osfr-tools/src/mining.rs`), and a private
real-engine probe crate. Private proof:
`lab/reports/harvesting-border-compatibility-20261009T1402Z/` (`geometry-model.py` + log,
`geometry-probe/` + log, `proposed-padded-shape.json`, `proof-manifest.json`, `final-hashes.txt`).
No source, content, UI, process or save was edited; the frozen Mining run `ef85` was not touched.

## Verdict

The original fall machinery structurally excludes **index 0 on both axes**: `checkFalls` never
scans column 0 or row 0, and `addColumnFall` never marks row 0 to fall. The current v1 shape
(cols 0..9 / rows 0..8 playable, drop row 9) exposes index 0 as playable, and the real engine
uses it: it accepts a chain whose tiles are entirely in absolute row 0, and a row-1 chain moves a
tile that started in row 0 down the column. A minimal padded shape (11x11 with row 0 and column 0
set to `0`, the same 10x9 playable rectangle at x1..10 / y1..9, and the drop row at y10) is
already accepted by the engine and keeps index 0 inert, matching the original loops.

## Original AS2 evidence

- `ServerGrid.as:57-58` `nMapHeight = aMyGrid.length; nMapWidth = aMyGrid[0].length` - the grid
  comes from the level template (`GameServer.as:279 SetGridShape(mGridTemplates[gridId])`).
- `ServerGrid.as:536-548` `checkFalls` scans columns `var _loc8_ = this.nMapWidth - 1;
  while (_loc8_ > 0)` - **column 0 is never scanned**.
- `ServerGrid.as:563-600` the per-column row scan is `_loc6_ = this.nMapHeight - 1;
  while (_loc6_ > 0)` - **row 0 is never scanned**; an empty cell triggers `regenTile` plus
  `addColumnFall(_loc6_, _loc8_)`.
- `ServerGrid.as:655-673` `addColumnFall` marks tiles with `_loc3_ = _nPosY - 1;
  while (_loc3_ > 0)` - rows `1 .. nPosY-1`; **row 0 is never marked to fall**.
- Client copies use the same bounds: `Grid.as:1103` / `Grid.as:1119` (`checkFalls`) and
  `Grid.as:1231-1246` (`addColumnFall`), plus `Grid.as:178-189` (`Failure()` also loops `> 0`).
- Shape grammar: `ServerGrid.as:352-368` maps `0 = TILE_EMPTY`, `1 = TILE_BUILDREGULAR`,
  `2 = TILE_BUILDANY`, `3 = TILE_BLOCKER`, `4 = TILE_DROPZONE`.

Consequence: under the original semantics a tile in index row 0 (or index column 0) can never
move, so any reconstructed playable cell there is a behavioural addition. Whether the original
template literally carried a `0` border row/column or inert tiles is not recoverable: the template
file `MatchThreeLevelData.txt` (`GameServer.as:77,109,296-325`) is not present in the extracted
scripts or the lab, so the original geometry remains **unknown / reconstruction**.

## Current geometry and engine rules

- Current package shape (10x10): rows 0..8 = `1`, row 9 = `4`.
  Derived bounds (`mining.rs:960-1027`): `left 0, top 0, width 10, height 9`, drop row 9.
- `mining.rs:1198-1247` `gravity` compacts and refills only inside the derived bounds, so with
  `top = 0` and `left = 0` row 0 and column 0 are full participants; `in_board` (`:1443`) uses the
  same bounds, so index 0 is chainable.

## Demonstrated bounded mismatch (real engine, private probe)

Four private fixture runs against the linked engine (`geometry-probe.log`):

| case | chain | result |
| --- | --- | --- |
| `current_row_zero_chain` | three `404`s at absolute (6,0),(7,0),(8,0) | **accepted**, score 400, no reshuffle |
| `current_row_one_chain` | three `401`s at (3,1),(4,1),(5,1), one `402` above at (3,0) | accepted; column x3 afterwards: rows 0-6 = `405` refills, **row 7 = `402`** (the row-0 tile was moved down), row 8 = `403` |
| `padded_row_one_chain` | same row-1 chain on the padded shape | accepted; absolute row 0 stays all `0` (`value_at_3_0 = 0`), refill lands at absolute row 1 |
| `padded_row_zero_chain` | the same row-0 chain on the padded shape | **rejected**: `mining chain outside board or repeated tile` |

Model cross-check (`geometry-model.py`): for a row-1 hole the AS2 `addColumnFall(1, x)` marks zero
rows and row 0 keeps its tile, while the Rust gravity model compacts the column and refills the
top rows - the same divergence the engine probe shows.

## Proposed minimal padded shape (private proposal; not applied)

`proposed-padded-shape.json`: width 11, height 11, row 0 and column 0 all `0`, playable `1`s at
x1..10 / y1..9 (90 cells, identical 10x9 playable rectangle), drop row 10 = `4` at x1..10.
Derived bounds: `left 1, top 1, width 10, height 9`, drop row 10. Engine acceptance and padding
inertness are both shown by the probe above. Only `width`, `height` and `shape` change; `selectable`,
`collectable`, `tiles`, `goals`, `outputs`, `duration_seconds`, `seed`, all spawn percents and
`meter_id` stay untouched. No diamond or other original playfield-shape fidelity is claimed - this
is the smallest border that satisfies the AS2 `x > 0 / y > 0` rule with the current playable area.

## Limits and hand-offs

- Original grid dimensions/border values remain unknown (template file absent); the padding above
  is a reconstruction proposal, not a recovered original.
- The root's label "module 2727" could not be matched to any file by hash search (checked the
  Harvesting package, the decompiled tree and the crate sources); the geometry source used here is
  the current package's `harvesting.json` `280f5cd7…`.
- Missing tile maps are piece028's separate line (`tests/harvesting_tile_registration.rs`
  `1f451b0d…`, `src/restoration/piece_028.rs` `766ac778…`) and are not duplicated here.
- The frozen Mining run `ef85` and any live content/save/process were not read or modified by the
  probes; every fixture is a private OS temp directory.

## Commands

```
python lab/tools/universal-modder-env/Scripts/python.exe \
  lab/reports/harvesting-border-compatibility-20261009T1402Z/geometry-model.py

env CARGO_HOME=lab/tools/cargo RUSTUP_HOME=lab/tools/rustup \
  lab/tools/cargo/bin/cargo.exe run --offline --quiet \
  --manifest-path lab/reports/harvesting-border-compatibility-20261009T1402Z/geometry-probe/Cargo.toml
```
