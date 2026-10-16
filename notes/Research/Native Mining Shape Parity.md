# Native Mining Shape Parity

Lead investigation from original scripts and the retained native125556 snapshots. No source/content/save changes made for this finding.

Original `core/ServerGrid.as:357-361` maps shape1 to `TILE_BUILDREGULAR` and shape2 to `TILE_BUILDANY`. `core/ServerTileGenerator.as` chooses BUILDREGULAR from the regular randomizer (selectable/hazard/wildcard), while BUILDANY includes collectables. Rust `initialize_board` currently calls `random_tile(run,spec,true)` for both1 and2. Its module contract itself describes1 as regular random and2 as any random. This is a generator parity error; concrete level geometry, seeds and percentages remain reconstructed.

The unedited native125556 board initially contained stationary ores at bottom playable row10, cols8/10. Its six accepted chains moved one upper ore fromcol9/row6 to row8 but no new ore reachedrow10; therefore zero collected does not by itself prove the callback for moving bottom ore is broken. The original idle tile does not enter fall merely from a dropzone flag. Starting regular cells should not contain those idle ores; collectable replacements remain allowed.

Proposed narrow repair after the separate dead-board reshuffle repair: shape1 uses `allow_collectable=false`, shape2 and refill queues retaintrue; preserve quotas, configured seed, percentages, output definitions and save branches. Test-first checks must force the collectable branch to prove shape1 refusal/shape2 acceptance and preserve refill availability. Native whole-round acceptance remains required.

Receipt: `../lab/reports/mining-shape-parity-lead-20261009T133418Z/receipt.json`. [[Native Mining Dead Board]] and [[Reviews/Native Mining 125556]].
