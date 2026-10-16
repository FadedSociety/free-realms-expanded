# Mining clickable round feasibility

Status: harness rebuilt from the accepted source and the zero-powerup regression reproduced; the
clickable evaluation itself is **not yet run**, with the exact blocker and resume point recorded
below. October 9, 2026 (UTC). Lab-only: no shared content, save, source, UI or release change.

## Receipts so far

| Item | Value |
| --- | --- |
| accepted source pin | `src/mining.rs` sha256 `4abe6b07f8edf52798ce810cde752a58fb959c30fcb639675371a041e463eb46` |
| private harness copy | same hash, `exact_match=true` (`source-pin.txt`) |
| harness build | `harness-build.log` (warnings only, binary produced) |
| baseline content | `25370f04…` goals 15+12, seed 18, `collectable_percent` 10, duration 180, outputs `[]` |
| **regression run** | `regression-zero-powerup/feasibility.json` — **1127 chains, collected `[15, 14]`, phase `LevelComplete`**, 0.828 s compute — matches the previous accepted 1127-chain result exactly |

The regression used the corrected settling model (fall row `PLAYABLE_MAX = 10`, `OnLevelEventMsg 50`
only for a goal actually on that row, `OnIdleMsg` only for live in-rect `idle = false` cells) and the
reviewed planner for every chain.

## Blocker and resume point (clickable evaluation)

**Traced contract (accepted source + accepted tests, no invention).** The inbound click is
`OnActionMsg\t<value>\t<x,y,>`: `value` is 950 dynamite, 951 wild tile, 952 destruct, 953 shuffle;
the list is the pressed tile followed by the named tiles. Accepted vectors:
`OnActionMsg\t950\t3,4,3,5,3,3,4,4,4,4,` (dynamite, recovered neighbour order; a right-before-down
order is refused) and `OnActionMsg\t951\t3,3,4,3,` (wild tile: pressed tile plus one named
selectable; duplicates, overflow beyond the definition power and mixed tiles are refused without
board mutation). `has_move` in the same source treats any 950–953 tile on the board as a legal move,
so a board holding a clickable is never declared dead, while `collect_fall` refuses a clickable that
reaches the fall row. The engine validates every list, so the driver can attempt the recovered
shapes and must record refusals rather than assume acceptance.

The driver has no click input path yet, and the click-target generation must come from the recovered
AS2 routines rather than a hand-written shortlist. The engine's clickable surface in the pinned copy
is: definitions `950..=953` at `mining.rs:1054`/`:1059`, the refusal of a falling clickable at
`:1523`/`:1530`, and the `950..=953` handling at `:2002` — that last site is where the inbound click
message and its validation have to be read before any trial matrix is meaningful.

Resume steps, in order:

1. Read the `:2002` arm (and the AS2 `GameLevel`/`Grid` click routine it mirrors) to pin the exact
   inbound message, its coordinate/ordering rules and the full-power/pool counters.
2. Extend the private driver with that click input, tracing the click count, the fired powerup and
   any parser refusal for each input; no forged shortlists and no invented targets.
3. Private spec copies varying only `powerup_percent` at reconstructed rates, deterministic seeds,
   bounded to ≤1500 chains and ≤30 s total compute, 4 s per chain (model, not measured; true wall cap
   1 800 000 ms).
4. Report per trial: chains, collected, reshuffles, fall ACKs, clicks/fired powerups, phase, engine
   wall/remaining, stop reason — including failures — all labelled RECONSTRUCTED, never retail.

No parameter is selected, deployed or native accepted by this note.

## Actual clickable trials (accepted 4abe6b07 engine, private reconstructed copies)

`clickable_study.py` authored `powerup_definitions` (absent from the baseline package, so authoring
was unavoidable) with `{spawn_chance = rate, duration = 0, power = 2}` for 950–953 and set only
`spec.powerup_percent`, leaving goals 15+12, seed 18, collectable_percent 10, duration 180 and
outputs `[]` at the baseline. Chains came from the reviewed planner with the corrected settling
model; clicks used the traced `OnActionMsg\t<value>\t<x,y,>` shapes.

| Rate (RECONSTRUCTED) | Chains | Clicks attempted | Refusal (engine validator) | Phase | Wall | Collected |
| --- | --- | --- | --- | --- | --- | --- |
| 10 | 11 | 1 | `mining action tile mismatch` | Playing | 44 000 ms | `[0, 0]` |
| 25 | 5 | 1 | `mining action tile mismatch` | Playing | 20 000 ms | `[0, 0]` |
| 50 | 0 | 1 | `mining action tile mismatch` | Playing | 0 ms | `[0, 0]` |

An earlier revision with a five-tile 950 list was refused with `mining action exceeds its power`;
capping the list to the authored power 2 changed the refusal to `mining action tile mismatch`, which
is the engine demanding the exact recovered tile set for the pressed powerup. Receipts per rate:
`runs/powerup-*/clickable-run.json` and `host-transcript.jsonl`, plus `clickable-study.json` and
`clickable-study.log`.

**Result:** no clickable trial completed a round. The blocker is now a concrete validator contract,
not a driver gap: the second tile of each list must be the *recovered* target set (the 951 named
selectable and the 950 neighbour sequence the accepted tests pin), which the accepted vectors
demonstrate but which this private driver has not yet reproduced tile-for-tile. Rates,
definitions and every assertion here are authored/reconstructed, never retail.

## Run 2 (corrected generator, preserved alongside run 1)

Run 2 mirrors the original draw: the second tile is
`random_queue.pop % eligible_pool.len()` with splice, 950 scans DOWN(0,+1), UP(0,-1), RIGHT(+1,0),
LEFT(-1,0) and 951 draws the row-major idle selectable pool. A synthetic regression runs first and
passes (`950` draw 0 → DOWN, draw 1 → UP; `951` draw 5 with a two-tile pool → pool[1]). Clicks now
count toward the chain bound and each click is followed by `settle_falls` + `tick(4000)`, with a
per-run `settle_falls` reset and a 10 s `time.monotonic` guard per rate.

| Rate (RECONSTRUCTED) | Chains | Clicks accepted | Clicks refused | Stop | Wall |
| --- | --- | --- | --- | --- | --- |
| 10 | 16 | 2 | 1 (`mining action tile mismatch`) | `click-refused` | 64 000 ms |
| 25 | 5 | 1 | 0 | `error: malformed mining coordinate list` | 20 000 ms |
| 50 | 0 | 1 | 0 | `error: malformed mining coordinate list` | 0 ms |

Two further concrete validator anchors came out of run 2: a click whose drawn second tile does not
match the engine's expected target is refused with `mining action tile mismatch` (so the draw must
consume the same queue position the server consumed, which the client queue alone cannot always
prove), and the engine's `coordinates()` refuses an idle report carrying fewer than three tile pairs
— a one- or two-tile `OnIdleMsg` therefore aborts with `malformed mining coordinate list`. Both are
engine-contract facts to design against, not conclusions about clickable viability: no rate reached a
quota, no round completed, and rates/definitions remain authored/reconstructed.

## Correction: runs 1 and 2 are invalid-driver receipts, not feasibility evidence

Both run 1 and run 2 used a **wrong target generator**, so their refusal receipts say nothing about
whether clickables can carry a round. Preserved for audit: `clickable_study.run2-invalid.py`
(sha256 `1c9558ade08275ba…`), `runs/powerup-*/` (run 1), `runs/run2-powerup-*/` (run 2) and their
logs. Treat every rate row above as an artifact of a faulty driver.

What was wrong: non-951 powers fell through to an unfiltered adjacency list; 952/953 never used their
own eligibility rules; the draw ignored idle flags and the queue position discipline; and the driver
skipped `settle_falls` + `tick` after a click. The corrected private generator now mirrors the
original `core/Grid` eligibility under the pure server constraints:

| Power | Pool (power 2, pressed + one drawn tile) |
| --- | --- |
| 950 shatter | neighbours in DOWN(0,+1), UP(0,-1), RIGHT(+1,0), LEFT(-1,0) order, kept only when idle and the value is selectable 400–499, wildcard 200 or blocker 800 — no ore, no empty cell, no other clickable |
| 951 wild | row-major idle selectable tiles (400–499) |
| 952 destruct | row-major idle tiles whose value is selectable 400–499 or a blocker |
| 953 shuffle | row-major idle tiles with value ≥ 200, keeping ore and other clickables, excluding blockers, the 100 drop value and the pressed tile |

`SELECTABLE` is the server's `is_selectable` range 400–499, not the 401–405 test fixture subset.
Before any click the original client checks `random_queue.len() >= power` (2 here) and sends nothing
when the queue is short — the private driver returns `None` and records a short-queue failure instead
of inventing a default draw index. An empty pool is legal and yields the pressed tile alone (the
server does not refuse it). The draw is
`random_queue.pop % pool.len()` with splice, and the client queue alone is the only queue this driver
may consume — the server refills and advertises its own. `clickable_study.py` now runs only a
hand-computed synthetic regression by default — expectations are literals, never re-derived from the
helper: 950 with empty UP, ore RIGHT and non-idle clickable LEFT excluded; 951 row-major modulo; 952
selectable/blocker with 399 excluded; 953 keeping ore and another clickable while dropping blocker,
100 and the pressed tile; and short-queue refusal for all four powers — and prints
`hold: no engine trial until the piece-028 repair pin is confirmed`; the trial matrix requires an
explicit `--trials` flag and stays disabled until root confirms the pin.

**Dependency:** the engine's `coordinates()` refusing a one- or two-tile `OnIdleMsg` is a **true
production protocol bug**, not a design constraint — original `Grid.UpdateIdle` sends any non-empty
list. Piece 028 is repairing it to min-1 idle / min-3 chain (RED/GREEN). New trials must wait for
that accepted pin. Every rate, definition and pool rule here is authored/reconstructed, never retail.

## Run 3 (accepted 3cc6652b source, fresh private harness)

Engine pin: `src/mining.rs` sha256 `3cc6652b0da68542748c0af28fa9ef60abf5c6554e4b0f4dce5e4d3ced76ddfc`,
copied byte-for-byte into a new harness (`source-pin.txt`, exact match) on top of the root-confirmed
18-probe short-idle pass at `8b32`; the old `4abe` harness and runs 1/2 are preserved untouched.
Generator: the corrected `095637` logic (server `is_selectable` 400–499, the four pool rules,
`queue.len() >= 2` before any click, empty pool → pressed only) with only path/label resolution
changed. Parity and trials ran with a 10 s per-trial `time.monotonic` guard, 4 s tick per chain **or
click**, and the corrected single-ACK settling.

| Run | Actions | Clicks accepted | Clicks refused | Collected | Reshuffles | Fall ACKs | Phase | Wall |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| parity, zero powerup, 250 ms | 1127 chains | 0 | 0 | `[15, 14]` | 44 | 27 | **LevelComplete** | 281 750 ms |
| 10 % reconstructed | 450 (chains + clicks) | 170 | 0 | `[3, 6]` | 7 | 9 | Lost | 1 800 000 ms (wall cap) |
| 25 % reconstructed | 152 | 88 | 0 | `[0, 0]` | 0 | 0 | Lost | 608 000 ms (timer 0) |
| 50 % reconstructed | 80 | 61 | 0 | `[0, 0]` | 0 | 0 | Lost | 320 000 ms (timer 0) |

Every attempted click was accepted by the engine — the generator now matches the recovered
eligibility and queue discipline — yet no reconstructed rate reached the quotas under the 4 s per
action model: rate 10 exhausted the 30-minute wall cap, rates 25 and 50 exhausted the 180 s level
timer first. Receipts: `parity-zero-powerup/feasibility.json`, `runs/run3-powerup-{10,25,50}/` with
transcripts, `parity.log`, `clickable-study.log`, `clickable-study.json`. The parity receipt and the
study outputs live in `lab/reports/mining-clickable-run3-20261009T1749Z/`, while the per-rate run
directories were written by the generator under
`lab/reports/mining-clickable-round-20261009T1728Z/runs/run3-powerup-{10,25,50}/` because only the run
label, not the generator's base directory, was overridden. The root-permitted completion handshake
(`OnNextLevelMsg` → Won, `FRServer_GameOver` → results, `FRServer_GameClose` → active None) was **not**
exercised: the 250 ms parity run reached `LevelComplete` but the handshake steps were not driven this
turn, and no trial reached quotas without force. No content, save, source, UI or release change; all
rates/definitions remain authored/reconstructed, and no native or economic claim is made.

Cross-links: [[Research/Mining Whole Round Feasibility]], [[Research/Mining Starting Round Pacing]],
[[Research/Mining Restoration]].


## Lead lifecycle receipt inspection

The private run3 lifecycle receipt records1127 natural chains to collected[15,14]/LevelComplete, plain OnNextLevelMsg returning OnGameWonMsg/Won/completed_count1, followed by GameOver and GameClose clearing active while preserving completed_count1. A parameter-bearing NextLevel is refused. This is private-engine evidence, not a native results/return or gateway reward acceptance. Receipt: `../lab/reports/mining-clickable-run3-20261009T1749Z/lifecycle-handshake.json`; all modeled balance remains inactive.
