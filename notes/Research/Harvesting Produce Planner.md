# Harvesting Produce Planner

Implemented repair, 2026-10-09 (times UTC). `lab/tools/play-native-harvesting.py`
`df061eb8…` -> **`7d256a4ffe67fa23ab21ef55160c42829400d2aaeea35a5645b3cda801ddfcbb`**, pre-edit copy
preserved at
`lab/reports/harvesting-produce-planner-20261009T1448Z/play-native-harvesting.df061eb8.py`.
The Mining driver stays at `4c41fa64…` (native-active; untouched) and the adapter's
`MINING_DRIVER_SHA256` pin is unchanged. Private proof:
`lab/reports/harvesting-produce-planner-20261009T1448Z/` (`harvest-produce-probe.py`,
`probe-red.log`, `probe-green.log`, `harvest-planner.diff`, `self-test.json`, `dry-run.log`,
`refusal.log`, `make-dry-run-fixture.py`, `proof-manifest.json`, `final-hashes.txt`).

## Actual diff (planner rank only)

`harvest-planner.diff`: the whole edit is inside `plan_chain`'s nested `rank` (was
`(clears_adjacent_boolean, len, max_row)`). The DFS, node budget, chain rules, geometry
derivation, measured-layout gate, namespace checks, ACK/pause handling, focus and timeout
logic, and every input function are unchanged.

New priority (maximised), computed against the **derived** harvesting geometry:

1. `collect_now` - this path clears **every** blocker below an unmet produce, so that produce
   reaches the playable bottom row on the next gravity step (`bottom = geometry.top +
   geometry.rows - 1`);
2. then measured progress for the unmet goals: produce helped, blockers actually cleared below
   the produce, `-fewest blockers remaining`;
3. then length, then depth.

All cells come from `geometry.cells()` / `geometry.inside()` and the unmet set from
`unmet_collectables`, so there is no hardcoded Mining bound and no `x = 0` assumption; the
11x11 padded shape (playable x1..10 / y1..9, dropzone row 10) is handled by the same code.
No content, package, quota, seed, reward, save, UI or release value was touched.

## RED -> GREEN (padded 11x11 fixtures, four produce goals 601..604)

| check | `df061eb8` (RED) | `7d256a4f` (GREEN) |
| --- | --- | --- |
| `padded_geometry` (derive left 1 / top 1 / 10x9 / dropzone 10) | PASS | PASS |
| `collect_now_beats_length` (601 at (4,8), final blocker (4,9) vs a longer 402 chain under 602) | **FAIL**: chose `((7,5),(8,5),(9,5),(10,5))` | **PASS**: chose the chain clearing `(4,9)` |
| `collect_now_inside_bounds` | PASS | PASS |
| `completed_goal_skipped` (601 goal complete, collected[0]=15; unmet 603 targeted) | PASS | PASS |
| `nonchainable_blocker_no_false_collect` (604's last blocker cannot join a chain) | PASS | PASS |

The fixture check proves the planner picks the final blocker when such a chain exists; it does
not simulate the following gravity/queue and therefore claims no collection beyond the chosen
clearing path.

## Regression matrix (all on `7d256a4f…`)

| suite | result |
| --- | --- |
| Harvesting adapter `--self-test` | 13/13 checks, exit 0 (`self-test.json`) |
| Normal mock dry-run (synthetic state, real content shape) | `ok: true`, plan `[[2,1],[2,2],[2,3]]`, `below_collectable [2,0,601]` - identical to the 1337Z baseline (`dry-run.log`) |
| Normal mock measured-layout refusal | `ok: false`, same refusal text as the baseline (`refusal.log`) |
| Mining driver hash / adapter pin | `4c41fa64…` unchanged; `MINING_DRIVER_SHA256` still `4c41fa64…` |

## Limits

- Fixtures are synthetic and private; they prove the ranking change on padded geometry, not a
  live run. No native/UI acceptance is claimed.
- The measured-layout gate still refuses native input without a root-measured layout record, as
  before; this repair changed only the chain ranking.
- The pre-edit copy is retained for review; the Mining driver and its own probes were not
  re-run here because that file is byte-identical.

## Commands

```
python lab/tools/universal-modder-env/Scripts/python.exe lab/reports/harvesting-produce-planner-20261009T1448Z/harvest-produce-probe.py <adapter>
python lab/tools/universal-modder-env/Scripts/python.exe lab/reports/harvesting-produce-planner-20261009T1448Z/make-dry-run-fixture.py
python lab/tools/universal-modder-env/Scripts/python.exe lab/tools/play-native-harvesting.py --dry-run --state <fixture> --content <harvesting.json> --report-dir <dir>
python lab/tools/universal-modder-env/Scripts/python.exe lab/tools/play-native-harvesting.py --self-test
```
