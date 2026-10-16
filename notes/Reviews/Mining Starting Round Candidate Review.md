# Mining starting round candidate - independent review

Independent private review, 2026-10-09 18:06Z. Read-only on shared inputs; all new artifacts live
under `lab/reports/mining-starting-round-independent-20261009T1806Z/`. No production source,
content, save, UI, gateway economy or native client state was touched, and the author's note,
script, specs and transcripts were not edited.

## Pins

| Item | sha256 |
| --- | --- |
| accepted harness `mining-clickable-run3-20261009T1749Z/harness/src/mining.rs` | `3cc6652b0da68542748c0af28fa9ef60abf5c6554e4b0f4dce5e4d3ced76ddfc` |
| harness `harness/src/main.rs` | `6ac9fed291815fdd8f80e18ec27856e4cb09f38ae9634a36e9e2144ea0b6dcf9` |
| harness binary `mining-host.exe` (built 2026-10-09 17:49:41Z) | `5bc4e4224f00c2bb9dc9f416c24fe30561ec1a18cc533472a8f04e863dadd479` |
| baseline `opening-harvesting-native-20261009T142624Z/content/mining.json` | `25370f048ec87f12c30b38ac338d1585de3c04bbd2c080185981a1d4c8e33c8b` (unchanged before/after) |
| candidate spec `percent40_quotas6_6.json` | `24568d64f7d32dcd7e9b025c963e85333bc0895ab4cc0207844d0549626358f8` (my independent rebuild is byte-identical) |
| reviewed planner `lab/tools/play-native-mining.py` | `4c41fa64a972bb17ea9b390f029d19e9eab618f9568b1b0f285d8e3d6157b742` |
| settle/ACK source `drive_mining_feasibility.py` | `77b1b1b1e718adecd3213860fed7812bf4652035f69737ec4f3062bb50ff15c8` |
| author transcript `runs/percent40_quotas6_6/transcript.jsonl` | `07c77adcfe8dd31840ae902836754de380916c963ae31e26210c1bb2608ee93e` |

## Exact candidate diff

Recursive leaf diff baseline -> candidate: exactly three numeric fields changed. `per_reward` (1
and 2), `outputs []`, seed 18, `duration_seconds` 180, the 14x14 board with its 8x8 interior
playable rect, selectables 401..405, wildcard 5 %, powerup 0 % and the reconstructed-rule notice
are all field-equal.

1. `spec.collectable_percent` 10 -> 40
2. `spec.goals[0].quota` (value 601, `per_reward` 1) 15 -> 6
3. `spec.goals[1].quota` (value 602, `per_reward` 2) 12 -> 6

## Independent replay (all 17 checks true, exit 0)

`independent_acceptance.py` drove the pinned harness through the ordinary contract (begin, accept
18000001, OnConnectMsg, OnStartGameMsg, OnStartLevelMsg, planner chains, reconstructed
`OnLevelEventMsg 50` / `OnIdleMsg` settling, 4000 ms model ticks), then the ordinary completion
handshake. Evidence: `independent-run.json`, `run.log`, `transcript.jsonl`, `snapshots/*.json`.

- 85 chains, 12 fall ACKs, stop `phase-LevelComplete`; 85 x 4 s = **340 s** under the author's
  model (inside the 5-10 minute window).
- Pre-close Playing snapshot: `chains 84`, `collected [6,5]`, `remaining_ms 538000`,
  `wall_ms 336000`, `reshuffles 9`, `score 58100`.
- Pre-close LevelComplete snapshot, captured before any completion message:
  `phase LevelComplete`, `collected [6,6]`, `remaining_ms 546000`, `wall_ms 340000`,
  `reshuffles 9`, `chains 85`, `score 59200`, `completed 0`. These are the values the author's
  receipt read as `null` because it sampled state after close.
- Handshake, observed exactly:
  1. `OnNextLevelMsg` -> reply `OnGameWonMsg`, completed flag true, phase `Won`, `completed 1`,
     `end_sent false`.
  2. `FRServer_GameOver` -> changed true, `end_sent` true, phase stays `Won` (results presented;
     the harness decodes only family 39 / sub 14 text replies, so the results payload itself is
     not text-visible).
  3. Replayed `FRServer_GameOver` -> changed false (presented flag consumed; idempotent no-op).
  4. `FRServer_GameClose` -> `active null`, `completed 1`.
- `collected [6,6]` meets both quotas naturally; no score, reward, state or content message was
  forced.

## Author transcript cross-check

The author's transcript (539 entries: 85 chain / 85 tick / 274 state / 93 msg) recovers first
Playing `[0,0]`/180000 ms, first LevelComplete `[6,6]` at `wall_ms 336000`, last pre-handshake
LevelComplete at `wall_ms 340000`, `Won` `completed 1`, and Close `active null`/`completed 1`. My
independent replay matches the recovered pre-close and lifecycle states field for field
(`author_*_matches` checks), so seed 18, the reviewed planner and the same ACK model reproduce the
selected preset deterministically.

## Findings

- Accepted for the private harness: `percent40_quotas6_6` is playable end-to-end, meets the 6+6
  quotas at 85 chains / 340 s model (about 5.7 min), and completes the ordinary
  Won -> results -> close lifecycle with `completed 1` and no residue.
- Exact scope: three-field least-change candidate among the three tested even presets. The
  corrected author note (`8C99A9FD5DA3F91ADCC7F1B0CADCABD33F66D5C51C66B45C8A4D3C10956D47F3`)
  now states the same limit - not globally smallest, not an optimal rate - and no longer contains
  the stale "not run here" sentence; root reports the prior revision preserved in its package
  preflight report.
- Root's staged package `opening-mining-starter-20261009T180011Z/content/mining.json`
  (`24568d64...`) is JSON-deep-equal to this accepted candidate, so that package carries exactly
  the reviewed spec. It was not launched or deployed in this review.
- The only `fs::write` in the harness source is `write_content()` (`mining.rs:1014-1032`), a
  content authoring helper that `main.rs` never calls; the drive path used here is read-only.

## Limits (no native acceptance)

- The 4 s per action figure is a reconstructed model, not a measurement; `remaining_ms` is
  maintained by chain time bonuses and 4000 ms model ticks inside the harness, not by native play.
- No native client, gateway economy, reward grant, save/restart or release binary was exercised.
  Percentages, quotas and outputs remain reconstructed tuning; this is private-harness evidence
  only.
- Odd quotas 3/5 remain inadmissible (`quota % per_reward != 0`, `src/mining.rs:1117-1120`); the
  odd-run receipts stay historical audit artifacts.

Artifacts: `lab/reports/mining-starting-round-independent-20261009T1806Z/` -
`independent_acceptance.py`, `independent-run.json`, `run.log`, `transcript.jsonl`,
`snapshots/{playing-initial,playing-last-before-completion,levelcomplete-preclose,after-close}.json`,
`content/percent40_quotas6_6.json`, `final-hashes.txt`.
