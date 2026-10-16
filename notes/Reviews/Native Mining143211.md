# Native Mining143211 (read-only audit)

Run `lab/runs/client-baseline-20261009T143211Z`; frozen gateway EF85
`ef85c0d9d05475f7d8c2bb5b3d9f960eeaf6b911a16ee7731b4a0720652ac32b` (planner
`4c41fa64…`); branch state file
`lab/state/minigame-round-native-20261009T142813Z/character.json`. No UI, source,
save, content, release or process edit by this audit; writes are this note and
`lab/reports/native-mining-143211-review-20261009T145407Z/`. All times UTC
(`stat` mtime via `datetime.fromtimestamp(mtime, timezone.utc)`).

## Verdict

Final failed run confirmed: 21 accepted chains, score 15,200, zero ore, one
reshuffle; the planner then refused with no legal chain. Frame capture, CRC3,
reassembly, whole-state conservation, inputs, two-recording exit qualification and
shutdown all re-derive cleanly. **No native win, no ordinary result Close**; the
world screenshot is *after* both recordings stopped, exactly as
`exit-qualification.json` states.

## 1. Frames / CRC3 / reassembly (AUDIT_OK=true, exit 0)

- Manifest 8,406 frames = 8,379 gateway + 27 login; 0 length/SHA-256 mismatches,
  0 unlisted/missing, 406,139 wire bytes.
- CRC length 3 both ways; 8,377 non-handshake gateway frames CRC3+opcode verified,
  0 decode errors.
- Reassembly complete: client→server 3,427 messages + 12 exact duplicates,
  server→client 169; 3,652 applications; one created-self 154,395 B
  `09d96f77ecb376dba268ffaef953a6bd30bd816c927b2b8738900ccf8c161285` (equal to
  `analysis/actual-created-self.bin` and the analyzer proof).
- `analysis/gateway-decode-proof.json` fields (gateway 8,379 / login 27 /
  crc 8,377 / length 3 / apps 3,652 / self 154,395 / 09d96f77 / complete): 8/8
  equal mine (`audit.log`, `audit-report.json`).

## 2. Conservation and the failed board

- `character-before.json` `fdf9ab62145e1b37f47dd3f634b6064ce306c7598fc775d8a23ba388eebbbfd3`
  (mining next 18000004, active null) → `character-after.json` =
  `character-after-confirmed-world-exit.json` = branch state file
  `e8c40a2e6781d38041d5b55511fc838c2111d507e81a99b447a54622730f1ea2`
  (next 18000005, active null, completed 0). Only diff path:
  `/gameplay/mining/next_instance_id`.
- Progression (paused true, phase Playing, end_sent false throughout): initial
  0/0; batch1 7/6,200; batch2 14/10,800/1 reshuffle; batch4 and chain21-no-move
  21/15,200/`[0,0]`/1 reshuffle (`wall_ms` 537,349 at the final paused snapshot).
- Independent refusal on the captured board (my own import of planner 4c41):
  `no legal chain on the board (zero valid path; budget 50000, visible unmet
  ores=1)`; all 196 idle flags true; wildcard at col 3,row 3 = 200 and the only
  ore at col 3,row 4 = 602 (the naive false-positive pair). Driver
  `planner-batch4/driver-summary.json` carries the identical refusal with no
  input receipts in that directory.
- `piece_028` `has_move` work is a separate root-owned fix and is not audited here.

## 3. Exit qualification and media chronology (visual + timestamps)

- First recording: ffmpeg log 14:33:52 → stop file 14:48:05.462, mkv/audio/json
  finalised 14:48:06 (audio 853.681 s). Stopped while the board was paused after
  the first dev-chat exit that did not cancel.
  `FAILED-EXIT-planner-world-exit.png` 14:48:05 viewed: still the Mining board at
  score 15,200 (quota 0/15, 0/6) — **not** the world.
- Second recording: ffmpeg log 14:48:34 → stop 14:49:25.278, finalised 14:49:25
  (audio 51.390 s); captures the unpause + dev-chat `realm exit` cancellation.
  `planner-cancel-try-again.png` 14:49:03 (inside the recording) viewed:
  **TRY AGAIN! / Mining**. No ordinary result Close.
- `planner-after-cancel-settle.png` 14:49:57 viewed: Darkthorne back in the world
  — created **after both recordings stopped**, so no world-inside-recording claim.
- `exit-qualification.json`: first reporting stopped-while-paused, second the
  realm-exit cancellation, world after both, `active_cleared=true`,
  `native_win=false`; consistent with every timestamp and screenshot above.

## 4. Inputs and shutdown

- 79 input receipts; helper `b4c5acfa4a013e3c80c78170b66792edeb011c6d0a3d16e5f901d0ff9dddf754`
  constant. Last actions: 14:43:23 drag-b3 + pause; 14:47:36 dev-chat realm exit
  (first, did not cancel); 14:48:58 pause-button click + dev-chat realm exit
  (second, cancelled).
- Manifest `client_stopped`/`login_stopped`/`server_stopped` true,
  `client_exit_code=4294967295`; PIDs 14740/32876/32440 not running and no
  FreeRealms/osfr process; `frames/live-status.json` closed, 8,406 written,
  0 pending, `error=null`; login stdout `session_ended / client_disconnect`;
  server log only saves/listening/session/world events (no stop/win event).

## Limits

- Visual claims are limited to the three screenshots actually viewed; the world
  state is outside both recordings by construction.
- Native win unproven; the zero-ore cause is delegated to the separate
  `piece_028` `has_move` fix.
