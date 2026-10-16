# Native Mining134930 (read-only audit)

Run `lab/runs/client-baseline-20261009T134930Z`; branch
`lab/state/mining-parity-native-20261009T134906Z`; frozen gateway
`lab/tools/osfr-gateway-after-mining-parity-ef85c0d9.exe`
`ef85c0d9d05475f7d8c2bb5b3d9f960eeaf6b911a16ee7731b4a0720652ac32b` (equal to the run manifest).
No UI, source, save, content, release or process edit by this audit; the only writes
are this note and `lab/reports/native-mining-134930-review-20261009T141912Z/`.
All times UTC; file times via `datetime.fromtimestamp(mtime, UTC)`.

## Verdict

Raw capture, CRC3 decode, reassembly, state conservation, exit provenance and
shutdown re-derive cleanly and match the root analyzer field for field. This run is
**not a Mining win and has no Mining reward**: 41 accepted chains, score 25400,
`collected=[0,0]`, three reshuffles, then a developer chat `realm exit` cancellation.
The world screenshot `parity-exit-result.png` is inside the recording; there is no
TRY AGAIN / result-Close claim anywhere in the evidence.

## 1. Frames, CRC3 and reassembly (re-derived, not trusted)

Independent re-derivation (my own re-parameterized copy of the established read-only
audit helper, run against the raw frames):

- Manifest lists 14,734 frames: 14,716 gateway + 18 login. Every listed frame exists
  with the recorded byte length and SHA-256 (0 mismatches, 0 unlisted, 0 missing),
  566,445 wire bytes.
- Session handshake: CRC length 3 both directions. All 14,714 non-handshake gateway
  frames pass the SOE 3-byte CRC32 check and decode with matching opcodes
  (`crc_and_decode_verified=14714`, decode errors 0; the two skipped frames are the
  opcode 1/2 handshake).
- Reassembly completes both directions: client→server 6,507 messages plus 35 exact
  duplicates, server→client 261, 6,836 application messages, no queued/partial frames.
- One created-self message: 154,395 bytes
  `09d96f77ecb376dba268ffaef953a6bd30bd816c927b2b8738900ccf8c161285`, equal to
  `analysis/actual-created-self.bin` (hash recomputed) and to the analyzer proof.
- Proof comparison `analysis/gateway-decode-proof.json` (`crc_verified=14714`,
  `crc_length=3`, `apps=6836`, `self_bytes=154395`, `self_sha256=09d96f77…`,
  `complete_reassembly=true`): all eight compared fields equal mine. `AUDIT_OK=true`,
  exit 0 (`audit.log`).

## 2. Whole-state conservation and progression

- `character-before.json` `aff0eeae85f21a4a92563b6f95ab9095d59fb757d1ebc2c7b4d3143d601ed8a5` →
  `character-after.json` = `character-after-world-exit.json` = branch state file
  `fdf9ab62145e1b37f47dd3f634b6064ce306c7598fc775d8a23ba388eebbbfd3`.
- Only diff path between source and final: `/gameplay/mining/next_instance_id`
  (18000003 → 18000004). `mining.completed=0`, `active=null`, no reward keys in the
  final mining state; inventory, quests, economy, profiles, encounter, opening,
  cooking, smithing and identity sections are byte-identical.
- Progression snapshots (chains/score/collected/reshuffles):
  chain1 1/1000/[0,0]/0 · batch1 6/3800/[0,0]/0 · batch2 13/7400/[0,0]/0 ·
  batch3 20/11800/[0,0]/1 · batch4 27/16000/[0,0]/2 · batch5 34/19800/[0,0]/3 ·
  collection-investigation 41/25400/[0,0]/3. Every snapshot phase `Playing`,
  paused true, `end_sent=false`, zero ore.
- Gateway log: `OnStopLevel=0`, `"phase":"Won"=0`, `"phase":"Lost"=0`,
  `TRY AGAIN=0`, `resultClose=0`; the only `rewarded` hits are the pre-existing
  encounter field `rewarded":[411]`.

## 3. Chronology, driver receipts and exit provenance

- Recording: `native-mining-parity.ffmpeg.log` 13:50:06; audio 572,312,072 B =
  1,490.396 s (48 kHz stereo f32) → media spans ~13:50:06–14:14:56. Stop file
  14:14:55.668; mkv/audio/json finalized 14:14:56.35–56.56.
- `parity-exit-result.png` 14:14:36.780 (1279x960) is **inside** the recording and
  shows Darkthorne back in the world (Cape Indigocaudron), viewed directly.
- The two titlebar-X clicks (14:13:36 click 420,26; 14:13:59 clicks 860,26 + 420,26)
  did **not** quit: the session kept saving, `parity-quit-confirm.png` (14:13:46) and
  `parity-quit-dialog.png` (14:14:10) captured the dialogs. The only exit action in
  the receipt stream is the 14:14:25 dev-chat receipt
  (`type realm exit` + Enter), i.e. `realm exit` cancellation, not a normal result
  Close.
- 148 input receipts; helper `b4c5acfa4a013e3c80c78170b66792edeb011c6d0a3d16e5f901d0ff9dddf754`
  and um source `d5aeb003…` constant; labels resume 41 / pause 41 / unlabeled 11 /
  drags; no quit-labelled receipt. Driver logs: chain1 5, batch1 17, batch2..6 23 lines.
- Shutdown: manifest `client_stopped`/`login_stopped`/`server_stopped` true,
  `client_exit_code=4294967295` (0xFFFFFFFF), PIDs 32876/23480/22160 not running and
  no FreeRealms/osfr process left; `frames/live-status.json`
  `state=closed, written_frames=14734, pending_frames=0, incomplete=false, error=null`;
  login stdout ends `session_ended / client_disconnect`.

## 4. Limits

- The analyzer binary was not executed by me; verification is my own re-derivation over
  the raw frames plus a field-by-field comparison with its saved proof, which is
  stronger for provenance.
- Visual claims are limited to the screenshots actually viewed; no TRY AGAIN or
  result-Close frame is claimed or present in the log evidence.
- The paused native snapshot for the race agent was not touched.
