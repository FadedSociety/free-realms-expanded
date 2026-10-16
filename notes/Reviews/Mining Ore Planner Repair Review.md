# Mining Ore Planner Repair Review (read-only)

Repair under review: private Mining ore planner, preserved `735f8ece…` →
current `4c41fa64…` (`lab/tools/play-native-mining.py`); harvesting adapter
`df061eb8…` (`lab/tools/play-native-harvesting.py`). No UI, source, save,
content, release or process edit by this review; writes are this note and
`lab/reports/mining-ore-planner-repair-review-20261009T143416Z/`.
All times below are UTC (`stat` mtime converted with
`datetime.fromtimestamp(mtime, timezone.utc)`).

## Verdict

Repair verified as a **strategic-rank-only change**. RED→GREEN reproduces on the
44 captured missed boards plus the immediate ore-to-bottom fixture; the input,
ACK, timeout, focus and guard paths are unmoved; no quota, seed, content, save
or Rust source file was edited. Actual native win remains unproven (root owns the
native run).

## 1. Diff is one hunk, rank only

`git diff --no-index` preserved 735f vs current 4c41: **1 hunk, 34 insertions,
2 deletions**, entirely inside `find_path`'s local `rank()` (`driver-planner.diff`,
`8bbdaadee94d79237f4cba73b0b85293c8c26fa82c2e7bc7a6818410d0a4a911`). Every other byte of the 47,535-byte driver is
identical, so the DFS budget/order, `chainable` guards, idle flags, ACK/pause
ordering, timeout and cleanup code, focus handling and receipt writing are all
unmoved by this repair.

- Old rank: `(any tile directly under an unmet ore, len(path), max row)`.
- New rank: `(collect_now, helped, cleared_total, -remaining_min, len(path), max row)`.
- `collect_now` semantics (reviewed line by line): ores come from `unmet_ores`,
  which skips any collectable whose goal quota is already met (goal filter); for
  each unmet ore it counts path tiles in the same column below it
  (`row > ore_row`, path tiles are idle, in-bounds, selectable/wildcard — the
  safe-input rules unchanged) and compares against **every non-zero cell** in
  that column down to the last playable row. `collect_now` is true only when a
  path clears all of them; non-chainable blockers or ores below keep
  `remaining > 0`, so there is no false "immediate collect" claim.

## 2. Reproduced probes

- Ore-planner probe (`ore-planner-probe.py` from the repair bundle) re-run:
  preserved 735f → `captured_missed_boards FAIL: 44 of 44 boards still miss`,
  `fixture_final_blocker FAIL`, failed=[captured_missed_boards,
  fixture_final_blocker]; current 4c41 → 44/44 improved, fixture PASS,
  no-progress fallback PASS, failed=[] (`probe-735f.log`, `probe-4c41.log`).
- Mocked transition harness (`mock-transitions.py`, 14:xx): 8/8 PASS on **both**
  revisions (ack_before_pause, terminal_no_pause, cleanup_release,
  cleanup_foreground_guard, idle_required, deadline_no_input, budget_clean_stop,
  budget_during_ack) — the repair touches none of these paths
  (`mock-play-native-mining.log`, `mock-4c41fa64.log`).
- Mocked timeout probes (`mock-timeout-probes.py`): 4/4 PASS on both revisions
  when run with the universal-modder interpreter
  (`timeout-um-play-native-mining.735f8ece.log` for 735f,
  `timeout-um-play-native-mining.log` for 4c41). The same harness under the
  wiki-env Python fails 3/4 identically on **both** revisions with "the
  universal-modder Python environment is required for driving"; that is an
  interpreter-environment error, not a regression
  (`timeout-play-native-mining.735f8ece.log`,
  `timeout-play-native-mining.log`).
- Driver self-tests: mining 14/14 checks `ok:true`; harvesting 13/13 `ok:true`,
  with `frozen_sha256=4c41…`, `previous_task_pinned_sha256=735f…`,
  `previous_copy_matches=true` (`self-test-mining.log`, `self-test-harvesting.log`).

## 3. Harvesting adapter df06 is pin-only

`play-native-harvesting.py` `df061eb8…` pins `MINING_DRIVER_SHA256 = 4c41fa64…`
and `MINING_DRIVER_PREVIOUS_SHA256 = 735f8ece…`, verifies the preserved copy and
the reused mining-driver interface at import (self-test passes). Its own planner
`rank()` (lines 481-483) still uses the old immediate-ore form and was not
converted to the new strategy. The adapter's previous revision was not preserved,
so "pin-only" is supported by its docstring, the frozen/previous pin fields and
the unchanged planner block rather than a stored byte diff. No harvesting
behaviour claim is made here.

## 4. Scope / non-edits

- `lab/tools/play-native-mining.py` `4c41fa64…` mtime 14:26:45Z and
  `lab/tools/play-native-harvesting.py` `df061eb8…` mtime 14:27:33Z; repair note
  `Mining Mouse Ore Strategy Repair.md` `8eeb261f…` 14:30:04Z.
- Unchanged: content `mining.json` `25370f04…` (mtime 12:09:12Z, before the
  repair), `src/mining.rs` `6c59ae0c…` (13:39:34Z), strategy inputs
  `strategy-probe.json` `3bc7630b…` (14:21:28Z) / `server-save-timeline.json`
  `ef635f73…` (14:19:50Z).
- Concurrent, not part of this repair: `src/gameplay.rs` changed 14:29:19Z and a
  new `lab/state/minigame-round-native-20261009T142813Z` appeared; noted only.

## 5. Limits

- No native run: the strategic improvement is proven on captured boards and
  mocked seams only; an actual native win is unproven.
- The 44 captured vectors come from the earlier 1419Z strategy extraction; the
  probe compares against that extraction's `distance_value`, not against a fresh
  server save.
