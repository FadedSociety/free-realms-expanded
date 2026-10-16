# Mining starting round pacing

Lab-only tuning study, October 9, 2026 (UTC). Varies **only** the authored
`spec.collectable_percent` on private spec copies; goals 15+12, seed 18, duration 180, empty outputs,
the 14x14 map and its8x8 playable rectangle and both goal values 601/602 are preserved byte-for-byte. No shared content, save,
source, UI or release file was touched, and no quota, chance, seed or state was forced.

Owned artifacts:

| Path | Role |
| --- | --- |
| `lab/reports/mining-pacing-study-20261009T1716Z/run_pacing_study.py` | study driver: builds the private copies, runs the harness per parameter |
| `…/specs/mining-percent-10.json` … `-50.json` | private copies; only `collectable_percent` differs |
| `…/runs/percent-*/feasibility.json`, `driver-stdout.log` | per-parameter receipts (old feasibility receipts untouched) |
| `…/pacing-study.json`, `…/pacing-study.log` | consolidated table and run log |
| this note | probe table, candidate path, limits |

Engine pin: frozen `src/mining.rs` sha256
`038a135d64fe0c3be6a8e57eabbe002e7f8b651566b0ca260098175abf16959e` (zero-powerup baseline; the
upcoming shuffle repair is unrelated at `powerup_percent = 0`). Harness: corrected ACK model,
forcing fall row `PLAYABLE_MAX = 10`, `OnLevelEventMsg 50` only for goal values actually on that row,
`OnIdleMsg` only for live in-rect cells the gravity pass left `idle = false`.

## Probe table (fresh engine per row, 4 s per chain reconstructed model, 1500-chain cap)

| `collectable_percent` | Chains | Collected | Reshuffles | Fall ACKs | Phase | Engine wall | Model wall | 5–10 min credible |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 10 (existing authored baseline) | 450 | `[3, 9]` | 28 | 12 | Lost | 1 800 000 ms | 1800 s | no |
| 20 (RECONSTRUCTED) | 450 | `[4, 5]` | 37 | 9 | Lost | 1 800 000 ms | 1800 s | no |
| 30 (RECONSTRUCTED) | 450 | `[13, 11]` | 35 | 24 | Lost | 1 800 000 ms | 1800 s | no |
| 40 (RECONSTRUCTED) | 436 | `[15, 12]` | 44 | 27 | **LevelComplete** | 1 744 000 ms | 1744 s | no |
| 50 (RECONSTRUCTED) | 394 | `[16, 12]` | 42 | 28 | **LevelComplete** | 1 576 000 ms | 1576 s | no |

Compute per row ≤ 0.35 s, total well inside the 30 s budget. The two completions are engine
`LevelComplete` phases (quotas met), not native `Won`, and no reward, Close or restart is claimed.

## Finding

40 % and 50 % complete the whole quota under the reconstructed 4 s-per-chain model, but they need
~26–29 minutes of that model — **all five parameters fail the credible 5–10 minute range**, and the
lower three never reach the quotas inside the wall cap. No parameter is selected or activated.

## Candidate path (not deployed)

If a starting-round pacing change is ever wanted, the only lab candidate is an **immutable private
copy** `specs/mining-percent-40.json` (or `-50.json`) referenced explicitly by path; the driver's new
`--content` argument pins that copy's sha256 in every receipt and refuses anything else. Deploying it
would require root-authored content, a shared-package decision and native acceptance — none of which
this study provides. The original baseline stays `collectable_percent = 10`.

## Limits and audit

The 4 s per chain figure is a reconstructed model, not a measurement; a real client may pace chains
faster, which would change the wall estimate but not the completion ordering. The study exercises the
server engine with planner-derived chains only — no native client, no physics, no capture parity. To
reproduce: run `run_pacing_study.py`; every receipt records the copy hash, the engine pin and the
per-run statistics used in the table above.

Cross-links: [[Research/Mining Whole Round Feasibility]], [[Research/Mining Restoration]],
[[../Design/MMO Worker Contract]].

Lead provenance clarification: every tested spawn rate, including the existing10-percent baseline, is locally authored reconstruction. No retail spawn percentage is established by this study. Historical capture/format facts remain separate.
