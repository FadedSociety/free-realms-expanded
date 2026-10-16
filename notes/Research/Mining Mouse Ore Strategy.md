# Mining Mouse Ore Strategy

Read-only efficiency diagnosis, 2026-10-09 (times UTC). Run
`lab/runs/client-baseline-20261009T134930Z`: **41 chains, 25,400 score, collected `[0,0]`,
3 reshuffles**. Driver reviewed at `735f8eceb1f14e4decbf95d3ae88c9ff55d389799cb8b0a06c6c47cf3f3db594`.
Private proof: `lab/reports/mining-mouse-ore-strategy-20261009T1419Z/`
(`extract-saves.py`, `strategy-probe.py`, `ore-fall-audit.py`, `aggregate.py` + logs/JSON,
`proof-manifest.json`, `final-hashes.txt`). No driver/source/save/UI/process edit. The public
engine event order belongs to the Mining review; this note uses only its public rules
(chain >= 3 orthogonal matching idle tiles, gravity compaction, collection at the bottom playable
row: `prototype/osfr-tools/src/mining.rs:1198-1247`, `:1249-1268`).

## Answer first: did an actual chain ever drop an ore?

**Yes - 9 ore down-moves were observed across the run**, and one ore walked from row 3 to row 9
(one step above the bin row). **No ore ever reached the bottom row**, so none was collected.
From `ore-fall-audit.json` (162 consecutive board pairs within a reshuffle segment):

| chains | move |
| --- | --- |
| 2 | 601 (6,3) -> (6,4) |
| 3 | 601 (6,4) -> (6,5) |
| 4 | 601 (6,5) -> (6,6) |
| 9 | 601 (6,8) -> (6,9) |
| 14 | 601 (4,3) -> (4,4) and 601 (4,5) -> (4,6) |
| (4 further single-row moves) | total 9 moves, 0 ore at row 10 in any state |

So the mechanism works: the driver's "clears directly below an unmet ore" boolean repeatedly
pushed the (6,*) ore down six rows. It never finished: at the last board before the engine's first
reshuffle the ore sat at (6,9) with exactly one blocker left at (6,10), and **no legal chain in
that state touched column 6 below the ore** - the board then reshuffled (save ordinal 79),
discarding that near-complete progress.

## Existing ranking vs a distance-to-bin priority

Method: the 171 `gameplay_saved` events in `server-stdout.log` were parsed into boards (166 with a
board, 162 consecutive pairs inside a reshuffle segment); the driver module was imported read-only
and its DFS/ranking was ported with the same rules and budget. The port reproduced the driver's own
`find_path` choice exactly on every cross-checked board (`planner_port_matches_driver: true`).
The candidate priority scores a path by ore progress only:
`(ores_helped, blockers_cleared_in_those_columns, -remaining_blockers_min, length, depth)` where an
ore is "helped" when the chain clears a tile in that ore's column below it, and `remaining` counts
the tiles still below that ore inside the playable area.

`strategy-summary.json`:

- **44 of 166 boards**: a legal chain existed that strictly beat the driver's chosen chain on ore
  progress. **32** of them are outright waste - the driver's chain cleared no cell under any ore
  while an alternative did; **12** are better-target cases where both advanced some ore but the
  alternative moved an ore closer to the bin.
- **40 board states** contained an ore with exactly **1** blocker left (one clear from the bin).
  The driver's chain cleared that last blocker in **0** of them, and in **0** of those states could
  any legal chain clear it either - the finishing chances had already been lost earlier, not at the
  moment the board showed them.
- Example: at 5 chains (saves 23-25) the driver advanced no ore at all while a legal chain reached
  one blocker remaining under an ore.

The driver's key `(clears_boolean, length, max_row)` treats every "clears" equally: it does not
prefer the ore closest to the bin, the path that clears the most blockers in one ore's column, or a
vertical run that removes several blockers at once. That is the concrete inefficiency this run
shows - 41 chains bought score but only 9 single-row ore descents and zero collections.

## Candidate (model only - no driver change made)

Rank candidate chains by `(ores_helped, blockers_cleared_under_ores, -remaining_blockers_min)`
before length/depth. On the captured boards this priority had a strictly better ore-progress choice
on 44 boards, including the 32 wasted chains; it is a heuristic model, not a proof of optimal play,
and it changes no quota, seed, reward or content value.

## Limits

- Boards are gateway snapshots (paused states), not every frame; 171 saves cover the 41 chains.
- The priority model reuses the driver's own DFS rules and is evaluated only on those snapshots;
  it does not simulate future tile queues, so "would have collected" is not claimed - only that its
  chosen chain advanced ore progress where the driver's did not.
- Public engine rules are cited from `mining.rs`; engine event ordering, reshuffle timing and
  public-engine semantics remain the Mining review's scope and are not duplicated here.
- No quota/seed reduction and no engine-race assumption was used.

## Commands

```
python lab/tools/universal-modder-env/Scripts/python.exe lab/reports/mining-mouse-ore-strategy-20261009T1419Z/extract-saves.py
python lab/tools/universal-modder-env/Scripts/python.exe lab/reports/mining-mouse-ore-strategy-20261009T1419Z/strategy-probe.py
python lab/tools/universal-modder-env/Scripts/python.exe lab/reports/mining-mouse-ore-strategy-20261009T1419Z/ore-fall-audit.py
python lab/tools/universal-modder-env/Scripts/python.exe lab/reports/mining-mouse-ore-strategy-20261009T1419Z/aggregate.py
```
