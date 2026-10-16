# Native Mining 125556

Independent read-only audit, 2026-10-09. Run `lab/runs/client-baseline-20261009T125556Z`;
branch `lab/state/mining-tile-native-20261009T124608Z`; release
`prototype/osfr-tools/target/release/osfr-gateway.exe` `7d5be63f…`; content
`lab/references/opening-mining-native-20261009T121330Z/content` (`reconstructed_rules: true`).
Private proof: `lab/reports/native-mining-125556-review-20261009T1327Z/` (`audit.py`,
`audit.log`, `audit-report.json`, `dead-board-dry-run.json`, `proof-manifest.json`,
`final-hashes.txt`, `make-manifest.py`). No UI, process, source, save or content edit; the only
execution was that read-only audit plus a dry-run on a copied snapshot.
All times below are UTC: file times via `datetime.fromtimestamp(st_mtime, timezone.utc)`, input
times from the recorded timezone-aware `time_utc` fields.

## Verdict

The raw-frame, progression, quit and shutdown claims re-derive cleanly, with two qualifications:
the board was **dead by legal-chain search** (not by game timer) when the batch stopped, and the
world-return screenshot was taken **after** the recording stop. Mining was **not completed**: no
ore was collected and no reward exists in the final state.

## Verified

1. **Frames / CRC / reassembly (re-derived, not trusted).** Manifest `598dea01…` lists 13,857
   frames (13,834 gateway + 23 login). Every listed frame exists with the recorded length and
   SHA-256 (0 mismatches, 0 unlisted files, 537,409 wire bytes). The session handshake gives
   CRC length 3 both ways; all 13,832 non-handshake gateway frames decode with the SOE CRC-3
   check and matching opcodes, 0 errors. Reliable reassembly completes both directions
   (client→server 6,204 messages with 18 exact duplicates; server→client 105), 6,364 application
   messages. One created-self message: 154,395 bytes
   `09d96f77ecb376dba268ffaef953a6bd30bd816c927b2b8738900ccf8c161285`, equal to
   `analysis/actual-created-self.bin` (hash re-computed) and to the analyzer proof
   `gateway-decode-proof.json` `9310d298…`, whose `gateway_frames`/`excluded_login_frames`/
   `crc_verified`/`crc_length`/`apps`/`self_*`/`complete_reassembly` fields all match mine.
2. **Release.** `osfr-gateway.exe` hashes to
   `7d5be63fd3cb1814e15211c07423ddf472d63f0f236410d7790c6b5b5d475f39`, matching the run manifest
   and `branch.json`.
3. **Branch provenance and conservation.** `branch.json` prepared 12:46:08Z from the 121834
   ending; the run's `character-before.json` == branch `source-character.json` ==
   `lab/state/mining-score-native-20261009T121834Z/character.json` (`a6adf508…`). After the run,
   `character-after.json` == branch `character.json` (`aff0eeae…`) and differs from the base
   **only** in `gameplay.mining`: `inventory`, `quests`, `combat`, `profiles`, `opening`,
   `encounter`, `smelting`, `forging`, `cooking`, `identity_fingerprint` and `character` are
   byte-identical to the source; `identity-before` == `identity-after` (`76c814e8…`).
4. **Six ordinary chains, zero ore.** Driver logs: `driver-chain1` recorded only `fg` + `rect`
   (the pre-fix parser stopped before any input), `driver-chain2` recorded `fg`, `rect`, `resume`,
   `drag-b8` (`702,618 → 746,618 → 746,662`), `pause`; `driver-batch1` recorded `fg`, `rect` and
   four `resume`/drag/`pause` cycles (`b9`, `h10`, `b6`, `b10`); `driver-dry-run` wrote nothing.
   With the recorded manual drag (`658,530 → 574 → 618 → 662` at 13:01:10.087602Z, chains 0→1,
   score 0→600, 7 changed cells all in column 7) that totals six chains.
   `character-after-batch1-stop` (13:14:57Z) and `-settled` (13:15:02Z) show
   `chains=6 score=3000 collected=[0,0] remaining_ms=176230/172329`,
   phase `Playing`, all 196 `idle` true, `reshuffles=0`, `end_sent=false`.
5. **Dead board, qualified.** `character-dead-board-paused.json` (13:19:34Z) re-runs through the
   driver planner as `no legal chain on the board … visible unmet ores=11`; the board holds a
   single wildcard cell, so no all-wildcard chain exists either. The server timer still showed
   `remaining_ms=172329` (~2.9 min), `wall_ms=1246534` — the board was deadlocked by legal-chain
   search, not expired by the timer.
6. **Ordinary quit → world.** `external-inputs.jsonl` ends with the recorded UTC entry
   `2026-10-09T13:19:35.195131+00:00` (`focus; move 420,26 ×3; click 420,26; move ×2; click 420,26;
   move ×3`). `mining-dead-board-quit.png` (13:19:38Z, 1279x960) shows the **TRY AGAIN! / Mining**
   transient; `character-after-dead-board-quit.json` and `character-after.json` (both 13:19:37Z) have
   `mining.active=None`, `completed=0`,
   `next_instance_id=18000003`. `mining-world-return.png` (1279x960) shows Darkthorne back in the
   world, but its mtime 13:20:09Z is **after** the recording stop (13:19:48Z) and the finalized
   mkv/audio (13:19:49Z), so the world return is not inside the recording — the last in-video frame
   is the TRY AGAIN transient.
7. **Recording and shutdown.** `native-mining-tile.stop` present (13:19:48Z);
   `native-mining-tile.mkv` 27,097,630 B and `native-mining-tile.audio.raw` 516,693,136 B finalized
   13:19:49Z (`native-mining-tile.json` records the video/audio pair). Manifest:
   `client_stopped`/`login_stopped`/`server_stopped` true, `client_pid` 9312,
   `client_exit_code` 4294967295, elapsed 1459.16 s; no `FreeRealms`/`osfr-gateway` process remains.
8. **Screenshots.** `mining-colors-fixed.png` (13:04:43Z) shows distinct per-value tiles after the
   sprite-index repair, score 600, both quota rows reading `Iron` with `0 / 15` and `0 / 6`
   (12 tiles ÷ `per_reward` 2) — the local reconstruction's quality label, per the root's
   diagnosis, and the arithmetic is my reading of the reconstructed content. `mining-intro.png`,
   `mining-after-settle.png`, `mining-dead-board-quit.png`, `mining-world-return.png` are all
   1279x960 (hashes in `final-hashes.txt`).

## Not claimed

- No mining completion, win, reward or quota progress: `collected=[0,0]`, `completed=0`, and every
  non-mining gameplay subtree equals the base save. The board died with
  `Required Iron 0/15` and `0/6` on screen.
- Displayed quotas/names come from the local reconstruction (`reconstructed_rules: true`); this
  audit is not native acceptance of Mining rules or balance.
- Chains were helper-driven mouse automation (driver batches plus one recorded manual drag); no
  unattended human play is claimed. `driver-chain1` shows the pre-fix parser refusing before any
  input, preserved as history.
- No ffprobe duration decode of the mkv, single run, and the world-return screenshot lies outside
  the recording window as stated above.

Cross-links: [[Native Mining Mouse Driver]], [[Native Mining Score 121834]],
[[Design/MMO Worker Contract]].
