# Mining Mouse Ore Strategy Repair

Implemented repair, 2026-10-09 (times UTC), from the 134930 diagnosis
([[Mining Mouse Ore Strategy]]). Driver `lab/tools/play-native-mining.py`
`735f8ece…` -> **`4c41fa64a972bb17ea9b390f029d19e9eab618f9568b1b0f285d8e3d6157b742`**;
pre-edit copy preserved at
`lab/reports/mining-ore-planner-repair-20261009T1425Z/play-native-mining.735f8ece.py`
(hash-verified). Harvesting adapter pin updated narrowly
(`lab/tools/play-native-harvesting.py` `df061eb8…`). Private proof:
`lab/reports/mining-ore-planner-repair-20261009T1425Z/` (`ore-planner-probe.py`,
`ore-probe-red.log`, `ore-probe-green.log`, `driver-planner.diff`, `self-test-mining.json`,
`self-test-harvesting.json`, `review8-green.json`, `timeout4-green.json`, `proof-manifest.json`,
`final-hashes.txt`).

## Actual diff (planner only)

`driver-planner.diff`: **+32/-2, entirely inside `find_path`'s nested `rank`** (lines 297-299).
Nothing else in the mining driver changed - DFS bounds (`MAX_PATH_LEN` 12, `DFS_NODE_BUDGET`
50 000), guards, idle, pause/ACK/terminal handling, focus and timeouts, and every input function
are byte-identical to `735f8ece…`.

New priority (maximised):

1. `collect_now` - this path clears **every** blocker below an unmet ore, so that ore reaches the
   bottom row on the following gravity step;
2. then measured progress for unmet goals: `ores_helped`, `blockers_cleared` (actual cleared
   cells **below** the ore, not the adjacent-cell boolean), `-fewest_blockers_remaining`;
3. then `length`, then `depth`.

The `ore_cells` set is built from `unmet_ores(active, content)`, so quotas are respected exactly as
before; no quota, seed, reward, content or package value was touched.

## RED -> GREEN (captured 134930 boards + one fixture)

| check | 735f8ece (RED) | 4c41fa64 (GREEN) |
| --- | --- | --- |
| 44 captured boards where the distance model beat the old choice (`ore-probe-red/green.log`) | FAIL 44/44 still miss (e.g. save 23 chose a 6-tile chain, progress `null`, best available `(1,1,-1)`) | PASS 44/44 choose a path with progress `>=` the best available |
| synthetic fixture: ore at (4,9) with its last blocker at (4,10) plus a longer non-collecting chain | FAIL chose `[(6,6)…(10,6)]` | PASS chose the chain containing **(4,10)** (immediate ore-to-bottom) |
| save-78 board (no ore-advancing path exists) | PASS | PASS - still returns a legal chain (no regression, no hypothetical-win claim) |

The 44-board check only compares the chosen path's measured ore progress against the best progress
available under the driver's own DFS rules on the same captured board; it does not simulate future
tile queues, so no collection or win is claimed beyond the chosen path clearing the remaining
blockers when such a path exists.

## Regression matrix (all on `4c41fa64…`)

| suite | result |
| --- | --- |
| Mining driver `--self-test` | 14/14 checks, exit 0 (`self-test-mining.json`) |
| Harvesting adapter `--self-test` | 13/13 checks, exit 0; `matches_frozen: true`, `previous_copy_matches: true` (`self-test-harvesting.json`) |
| 8 mocked review probes (`native-mining-driver-20261009T1309Z/mock-transitions.py`) | `failed: []` (`review8-green.json`) |
| 4 timeout/receipt probes (`native-mining-driver-20261009T1334Z/mock-timeout-probes.py`) | `failed: []` (`timeout4-green.json`) |

## Harvesting adapter pin update (narrow)

`lab/tools/play-native-harvesting.py` changed only its pin and its docstring reference:
`MINING_DRIVER_SHA256` `735f8ece…` -> `4c41fa64…`, `MINING_DRIVER_PREVIOUS_SHA256`
`47662040…` -> `735f8ece…`, `MINING_DRIVER_PREVIOUS_COPY` -> this task's preserved `735f8ece` copy.
No harvesting behaviour, geometry, planner or namespace logic changed.

## Limits

- Only the ranking changed; the DFS, budgets, guards and input paths are untouched, and the
  captured-board claim is limited to measured blocker progress on stored boards.
- The fixture is synthetic and private; it proves the immediate-collect preference, not a live
  run. Native/UI acceptance and engine event ordering remain outside this repair.
- 735f copy and the older 47662040 copy stay preserved for review (`1334Z` report).

## Commands

```
python lab/tools/universal-modder-env/Scripts/python.exe lab/reports/mining-ore-planner-repair-20261009T1425Z/ore-planner-probe.py <driver>
python lab/tools/universal-modder-env/Scripts/python.exe lab/tools/play-native-mining.py --self-test
python lab/tools/universal-modder-env/Scripts/python.exe lab/tools/play-native-harvesting.py --self-test
```
