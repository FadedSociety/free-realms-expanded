# Mining Focused Ore Strategy

Read-only study, 2026-10-09 (times UTC). Run `lab/runs/client-baseline-20261009T151307Z` (live;
CB0/da3b engine, mining driver `4c41fa64…`). Question: does the current rank
(`collect_now, helped, total_cleared, -min_remaining, length, depth`) spread effort across ores so
that no ore is ever finished, and would a focus-on-lowest-remaining-first rank collect more?
Private proof: `lab/reports/mining-focused-ore-strategy-20261009T1548Z/` (`focus-model.py` +
`focus-model.json`, `real-batch-metrics.py` + JSON, `focus-probe/` (Rust attempt), logs,
`proof-manifest.json`, `final-hashes.txt`). No driver/source/content/save/UI/release edit.

## Label first

The comparison below is a **labeled model, not native**: chain/gravity/collect/queue semantics are
ported from `prototype/osfr-tools/src/mining.rs` (`validate_chain`, `gravity`, `collect_fall`), and
every synthesized fall is tagged `model`. The planned Rust probe that would have driven the
**real** `Type8Engine` was built, but this host's Application Control policy blocks newly built
executables (`os error 4551`, log `focus-probe.log`), so it cannot run here; that limitation is
recorded and no native claim is made. Each strategy replays from an identical deep copy of the
captured state, preserving the same `rng`, seed, `tile_queue`, `random_queue` and `collected`
counters.

## Measured from the real run (no simulation)

`real-batch-metrics.json`, batches 2..11 (each batch executed 7 chains):

- **70 chains executed; only 13 (18.6%) had a path with a tile directly below an unmet ore**
  (`below_ore: true`); 57 were score-only chains.
- `collected` stayed **[1, 0]** from chain 35 through chain 70; `reshuffles` rose 2 -> 4 between
  those snapshots; score rose 24,700 -> 43,900.
- The one collected ore was banked before batch 6; nothing reached the bin afterwards despite 35
  further chains.

That is the measured efficiency picture: most chains did not touch any unmet ore's column, and the
board reshuffled four times.

## Model comparison (7 captured states x 2 ranks x 5-chain horizon)

States: `character-chain28.json`, `batch6..10/character-after-batch.json`, `character-chain56.json`.
Both ranks plan with the same DFS rules (3..12 tiles, 50 000-node budget) and the same fallback to
length/depth when no ore-advancing path exists.

| state | current: chains / score+ / collected+ | focus-lowest-remaining: chains / score+ / collected+ |
| --- | --- | --- |
| chain28 | 3 / 2600 / [0,0] | 3 / 2600 / [0,0] |
| batch6 | 3 / 1200 / [0,0] | **4 / 1600 / [0,0]** |
| batch7 | 2 / 1000 / [0,0] | 2 / 1000 / [0,0] |
| batch8 | 3 / 1400 / [0,0] | **4 / 1800 / [0,0]** |
| batch9 | 2 / 800 / [0,0] | 2 / 800 / [0,0] |
| batch10 | 4 / 2200 / [0,0] | 4 / 2200 / [0,0] |
| chain56 | 2 / 800 / [0,0] | 2 / 800 / [0,0] |

Neither rank collected a single ore in any horizon, and the focus rank was strictly better on only
two of seven states (one extra legal chain and +400 score each). The per-chain `progress`
(helped, cleared, -remaining) and `collect_now` flags are in `focus-model.json`.

## Verdict and recommendation

The "current rank spreads effort across ores" hypothesis is **not supported** by this capture set:
changing the tie order to focus-on-lowest-remaining produced no collection and only marginal extra
chains. The binding constraint in this run is **reachability of the final blockers** (no legal
chain includes them at the moments the snapshots show), matching the earlier 134930 finding where
40 near-complete ore states had no chainable last blocker.

Per the task rule, **no candidate driver file was created**: the required RED fixture plus a
comparison showing a real advantage did not materialise. Root keeps the current driver `4c41fa64…`;
any swap should wait for evidence from a board where an ore's last blocker is actually chainable
(the rank already prefers such collect-now chains - see [[Mining Mouse Ore Strategy Repair]]).

## Limits

- The comparison is a model port; the compiled engine could not be executed here (policy block
  recorded). Native/UI acceptance, event order and reward semantics remain outside this study.
- Horizon is 5 chains per state; longer horizons, other snapshots, or bounded lookahead could
  change the numbers, and the model stops when its port sees no chainable tiles or no progress.
- No quota/seed/reward/content value was altered, and the live run was only read.

## Commands

```
python lab/tools/universal-modder-env/Scripts/python.exe lab/reports/mining-focused-ore-strategy-20261009T1548Z/real-batch-metrics.py
python lab/tools/universal-modder-env/Scripts/python.exe lab/reports/mining-focused-ore-strategy-20261009T1548Z/focus-model.py
```

## Follow-up: faithful replay validation + 100/300-chain rank study (2026-10-09, labeled model)

Private proof: `lab/reports/mining-focused-ore-strategy-20261009T1548Z/horizon-ranks.py` +
`horizon-validation.json` + `horizon-ranks-{primary,extended,long300}.json` + logs +
`horizon-selftest.log` + `horizon-hashes.txt`. Still no driver/source/content/save/UI/release edit
and no native claim (Rust probe still blocked, os error 4551).

### Replay validation, batch7..13 (exact previous-batch deep copies, recorded chain paths)

- The old `validate-model.py` port matches batches 7..12 and mismatches **batch13 by score only**:
  actual 53800 vs model 53300 (-500 = `COLLECTABLE_SCORE`, `mining.rs:99/1275`). Batch13 chain 5
  collected ore 601 in column 8; the shortcut skipped `collect_fall`'s second gravity, but later
  chains' gravity re-converged board/tile_queue/rng, so only the -500 persisted.
- `horizon-ranks.py` adds the exact `mining.rs` `collect_fall` path (remove bottom ore, +500,
  second gravity/refill per reported fall, completion check) and keeps `gravity`'s true refill
  order (pop, then `len<50` -> +20). Result: **batch7..13 all match, 7/7 chains each, all eight
  fields** (`board,tile_queue,random_queue,rng,collected,chains,reshuffles,score`). This is the
  model used below and the first replay evidence that includes a real collection event.

### Rank variants over captured states (DFS 10k node budget, truncation instrumented)

| variant | rule |
| --- | --- |
| `current` | driver `4c41fa64` rank: collect_now, helped, cleared, -remaining, length, depth |
| `focus` | prior note's lowest-remaining-first order |
| `prop1` | engine-truth collects, one-away ores, then helped/cleared/remaining |
| `prop2` | bounded 2-ply over the 6 best `prop1` chains; adds reply collects + survival (no forced reshuffle through both plies) |

100-chain totals over the 11 captured states (601, 602 collected): `current [5,0]`,
`prop1 [5,0]` (identical choices everywhere), `prop2 [6,0]` (the extra is one ore at chain 99 of
batch13), `focus [1,0]`. Per-state tables in `horizon-ranks-primary.json` /
`horizon-ranks-extended.json`.

300 chains (same states, `horizon-ranks-long300.json`): `current [20,42]=62`,
`prop1 [20,42]=62` (still identical), `prop2 [22,32]=54`. The 2-ply lookahead loses 8 of 62
collections at 300 chains; the 100-chain +1 was horizon noise. `prop2` had fewer forced
reshuffles (e.g. 6 vs 8 on batch9/chain56 at 100 chains; 18-19 vs 20-21 on batch10/batch12/batch13 at 300 chains) but lower score and ~6x
nodes; fewer reshuffles is not the driver's objective.

Robustness: repeat run identical (28/28 end-state hashes); `prop2` with keep=12/reply 5000 gives
identical collections on all 7 primary states; DFS truncation 0 everywhere. Rank prediction vs
engine collect count was exact on every 100-chain decision and on 299/300 decisions per state at
300 chains; the misses are queue-refill ores landing on the bottom row, which the pre-chain rank
cannot see (engine-only, same for all variants).

### Verdict and recommendation

Model validated, but **no legal-path advantage**: `prop1` (engine-truth collections + one-away
ores before greedy helped) is exactly the current driver rank on all 11 captures, and bounded
2-ply either ties (100 chains) or loses collections (300 chains). Per the task rule **no
candidate driver file was created; no change**. Root keeps driver `4c41fa64...`; the live run was
paused at chain84/85 and only read.

One real semantic gap is dormant, not a candidate: the driver counts `TILE_BLOCKER` (800) cells
below an ore as remaining, while the engine skips blockers in gravity, so the engine can collect
where the driver's `collect_now` is false (RED/GREEN fixture in `--self-test`). The shipped shape
has no in-bounds blocker cells (codes 0x124, 1x64, 4x8) and all 11 captures have 0 blockers in
play, so it cannot fire in this content; revisit only if a future shape places 800 in bounds.

Limits: quota completion is unreachable in-horizon (27 ores needed; ~0.19 collections/chain at
300 chains), so this study compares collection rate and survival, not level completion; the model
remains labeled, not native.
