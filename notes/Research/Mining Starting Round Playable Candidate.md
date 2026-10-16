# Mining starting round playable candidate

Lab-only candidate study, October 9, 2026 (UTC). Three private reconstructed presets were authored on
the accepted `3cc6652b0da68542748c0af28fa9ef60abf5c6554e4b0f4dce5e4d3ced76ddfc` harness, each
changing only `spec.collectable_percent` and the two per-kind goal quotas. Seed 18, the14×14 board with an8×8 playable interior, duration 180, `outputs []`, the no-powerup definition set and every other spec field
were asserted byte-equal to the baseline (`25370f04…`) before each run.

Owned artifacts:

| Path | Role |
| --- | --- |
| `lab/reports/mining-playable-candidate-20261009T1755Z/run_candidate_study.py` | preset builder + bounded runner |
| `…/specs/percent40_quotas4_4.json`, `…/percent50_quotas4_4.json`, `…/percent40_quotas6_6.json` | executed even-quota private copies; odd-quota copies remain historical |
| `…/runs/<preset>/transcript.jsonl` | per-preset engine transcript; odd-quota trials reached load only, even-quota trials completed normally |
| `…/candidate-study.json`, `…/candidate-study.log` | consolidated receipts |
| this note | result, recovered constraint, admissible analogues |

## Result (even-quota analogues, executed)

| Preset | Percent (RECONSTRUCTED) | Quotas | Actions | Model duration | Lifecycle | 5–10 min |
| --- | --- | --- | --- | --- | --- | --- |
| `percent40_quotas4_4` | 40 | 4+4 | 48 | 192 s | LevelComplete → Won → GameOver → Close (active None) | no |
| `percent50_quotas4_4` | 50 | 4+4 | 46 | 184 s | LevelComplete → Won → GameOver → Close (active None) | no |
| `percent40_quotas6_6` | 40 | 6+6 | 85 | **340 s ≈ 5.7 min** | LevelComplete → Won → GameOver → Close (active None) | **yes** |

All three reached their quotas naturally (no forcing), the completed counter settled at 1, and the
ordinary recovered callbacks produced `OnGameWonMsg`, results and a cleared run. The script's
`selected` field is **`percent40_quotas6_6`** — the only candidate among the three tested even-quota presets inside the5–10minute model window. This study does not establish a globally smallest quota or optimal spawn rate. `collected`/`reshuffles`/wall read `null` in
the receipts because the handshake had already closed the run; actions and model duration are the
measured values.

## Historical: the requested odd quotas are refused at load (kept for audit)

| Preset | Percentage | Quotas | Engine result |
| --- | --- | --- | --- |
| `percent40_quotas3_3` | 40 (RECONSTRUCTED) | 3+3 | load refused: `invalid mining goals` |
| `percent50_quotas3_3` | 50 (RECONSTRUCTED) | 3+3 | load refused: `invalid mining goals` |
| `percent40_quotas5_5` | 40 (RECONSTRUCTED) | 5+5 | load refused: `invalid mining goals` |

No trial produced actions, quotas or a win; the host panicked at content load before the first offer.
Raw receipts preserved as `candidate-study.odd-quota-invalid.json` / `.log` plus the per-preset
transcripts, untouched next to the corrected even-quota study.

## Recovered constraint (source anchor)

`src/mining.rs:1115`–`:1120` validates goals as
`!collectable.contains(value) || per_reward == 0 || quota % per_reward != 0`. The baseline goals carry
`per_reward` 1 (601) and **2** (602), so the second quota must be **even**. The requested 3 and 5 are
therefore not admissible content for this engine: an odd second quota can never satisfy
`quota % per_reward == 0`. This is an engine contract, not a driver limitation — the harness, planner
and settling model were not even reached.

That was the initial invalid-trial disposition only. The lead subsequently authorized even-quota analogues4+4 and6+6; their executed results are in the current table above.

## Standing limits

All percentages, quotas and definitions in this study are locally authored reconstruction, never
retail balance. The 4 s per action figure is a model, not a measurement; no native client, gateway
economy, reward or save behaviour was exercised, and no shared content, save, source, UI or release
file was touched. Baseline preservation is recorded by hash equality in each preset build and by the
pinned `candidate-study.json`.

Cross-links: [[Research/Mining Starting Round Pacing]], [[Research/Mining Whole Round Feasibility]],
[[Research/Mining Restoration]].
