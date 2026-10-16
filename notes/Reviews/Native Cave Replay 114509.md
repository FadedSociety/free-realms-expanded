# Native cave replay 114509 independent audit

Read-only audit of `lab/runs/client-baseline-20261009T114509Z` (free-play native client,
migration-format cave replay), 2026-10-09. Reviewer owns this note and
`lab/reports/native-cave-replay-114509-review-20261009T1200Z/` only; no source, test, shared note,
state, UI or process was touched and no window was driven. Skills: RTK (normal commands), Ponytail
(one small self-contained audit script instead of a new harness), Caveman (terse report),
game-automation (read the capture/helper evidence rather than re-running the game), reverse (raw
frame/CRC derivation).

Verdict: the run is a real, finalized native session with intact inputs and a coherent
loss → second win → third entry → guide exit → world-return sequence. Every number below was
re-derived from the raw frames or the saved snapshots; the one claim the reviewer could not re-derive
from first principles is the recording's decoded duration (no ffprobe available), which stays labelled.

## Independently re-derived from the raw run

`audit.py` (my own implementation, stdlib only) re-reads `manifest.json` and `frames/*.bin` and prints
`AUDIT_OK = True` (`audit.log`, `audit-report.json`):

| Claim | Re-derived result |
| --- | --- |
| Frames / services | `manifest.json` 5,025 frames = **5,007 gateway + 18 login**; every frame file's length and SHA-256 matches the manifest (0 mismatches), no extra and no missing `.bin` (5,025 files; `live-index.jsonl`/`live-status.json` excluded) |
| CRC | handshake `crc_length = 3`, reply `3`, flag `0100`, max payload 512; my own `zlib.crc32(seed_le + content)` low-3-byte check passes on **5,005 / 5,005** non-handshake gateway frames, decompression terminates exactly and every decoded opcode equals the manifest opcode (0 errors) |
| Reassembly | complete: client-to-server 463 messages (3 duplicates), server-to-client 743, no queued frame and no partial fragment |
| Applications | exactly **1,528** application messages; exactly one server-to-client inner-12 created-self, 154,395 bytes, SHA-256 `816FB9EF0DBE1B818E98DE28A968C1E608B051B89C49E925182397BD136FF9A5` |
| Analyzer agreement | all eight values equal the saved `analysis/gateway-decode-proof.json` (5,007 / 18 / 5,005 / 3 / 1,528 / 154,395 / same self hash / `complete_reassembly true`) |
| Release | `prototype/osfr-tools/target/release/osfr-gateway.exe` SHA-256 `4D7226E0…` = manifest `server_executable_sha256` = `branch.json` `release_sha256`; the older `lab/tools` copies hash differently |
| Branch receipt | `branch.json` source `111125/character-after.json` SHA-256 `ECEBABE6…` re-hashed equal (original and copied `source-character.json`); `identity_sha256 76C814E8…` re-hashed equal (`identity.json`, `identity-before.json`, `identity-after.json`) |

## Sequence bindings (byte level)

`server-stdout.log` carries 26 ordered `gameplay_saved` encounter states: instance **2**
loading/fighting (player hp 1000 → 900) → `active=null next=3` (loss close) → instance **3**
loading/fighting (hp 1000 → 700) → `id=3/won hp=660` → `active=null next=4` (win close) → instance
**4** loading/fighting → `active=null next=5` (guide exit). Snapshots confirm each state:
`character-cave-full-health-entry` (id 2, full 150/150/150/500 enemies, hp 1000),
`character-cave-first-loss` (id 2/lost, hp 0), `character-cave-second-full-health-entry` (id 3,
hp 1000), `character-cave-won-held` (id 3/won, hp 660, all four enemies 0, `rewarded [411]`,
`next_instance_id 4`), `character-cave-third-full-health-entry` (id 4, full enemies, hp 1000,
`next_instance_id 5`), `character-third-guide-exit` / `character-cave-closed` /
`character-final-world` (`active None`, `world_stage 0`, hp 1000).

Two bindings re-checked in the raw decoded bytes:

* the loss close `character-cave-first-loss` follows helper stop
  `"health at or below200; stop before knockout"` at hp 140 with three minions dead and the boss at
  500 (`combat-helper-2.log`, `fight-dungeon-…114841….jsonl`); the win followed the two bounded
  second-fight batches (batch 1 "insufficient time budget", batch 2 `"dungeon defeated"`, hp 660,
  kills 4) in `combat-helper-retry.log`;
* the three client `(opcode 41, sub 109)` close packets carry the encounter instance id in the
  payload: frames/002371 (t 261.11 s) `…29 00 6d 00 … 02 00 00 00` → **instance 2**,
  frames/003707 (t 420.64 s) `… 03 00 00 00` → **instance 3** (the claimed real close binds the
  current instance), frames/004992 (t 590.91 s) `… 00 00 00 00` → world close. `application-0742`
  (frames/004605, t 584.06 s) is the interaction selection carrying i32 **101** right before the
  world-return messages, matching `src/encounter.rs` `const EXIT: i32 = 101`.

## State, screenshots and stop

Every one of the 15 `character-*.json` snapshots holds `coins 309`, inventory
`next_instance_id 52`, and the same 17 rewarded opening stages `910001–910017`; `won-held`
inventory/profiles/opening are JSON-equal to `character-reloaded-before-cave`, so the held save is
the reloaded checkpoint, not a mutated one. `character-before`/`reloaded-before-cave` start at the
copied world checkpoint `[-1896.0884, -40.125534, 416.4878]`, `active None`, hp 775.

Screenshots (`replay-*.png`, 30+ files) were inspected where they carry the claim:
`replay-win-score` shows **YOU WIN! The Last Best Hope**, `Opponents Defeated 4 – 0`, `Goals 1/1 –
5000`, `TOTAL SCORE 0`; `replay-third-guide-menu` shows the cave with the Gloam Overlord bar and the
Darkthorne radial offering **Start / Exit** at 1000/1000 hp; `replay-third-world` shows the outdoor
world at full health after the exit; `replay-loss-return` is the FreeRealms loading screen, i.e. a
return/load, not a result panel. The `Goals 5000 / TOTAL SCORE 0` display issue stays tracked
separately and is **not** claimed fixed.

Stop/finalization: `client_stopped`, `login_stopped`, `server_stopped` all true; client pid 6492;
`elapsed_seconds 625.953`; `native-full-cave-replay.stop` present; `launcher.log` ends
`Captured datagrams: 5025`; `native-full-cave-replay.mkv` 139,476,668 bytes with a valid EBML header
and `native-full-cave-replay.audio.raw` 214,474,200 bytes (= 558.5 s at 48 kHz stereo f32).

## Limitations and non-claims

* The fights were driven by a bounded input helper (`external-inputs.jsonl`, helper SHA-256
  `b4c5acfa…`; manifest `input_automation:false` refers to the launcher). The evidence supports
  "no save/health/reward edits and no teleport inside this run" — server log has zero
  `teleport`/`dev_teleport`/`relocation` strings and only the gateway wrote the state — but the run
  is not a purely manual playthrough.
* The previous teleported starting location belongs to the earlier `111125` run/branch source; this
  audit did not re-audit that run, and walked-terrain coverage here is limited to the recorded
  movement traces (positions evolve 285→289 m inside the cave), not a terrain-validation pass.
* The local 17-stage opening remains a reconstruction: this run demonstrates the migration-format
  cave replay, not historical Free Realms tutorial/quest completeness.
* Recording duration was not decoded (no ffprobe); validity rests on the EBML header, stop flag and
  non-zero track sizes. Manifest overlay/asset hashes (`935bc0d9…`, journal/character-select
  overlays) were read, not re-hashed from their source packages.
* One run only; no repeatability or multi-client evidence.

## Evidence

| Artifact | SHA-256 |
| --- | --- |
| `lab/reports/native-cave-replay-114509-review-20261009T1200Z/audit.py` | `92EC9526F41B59DF47822DCD0A07B3132573636EA0CA10C9A442395853FCD426` |
| `…/audit.log` | `11D6721B8C93EAEFA765593952C25520CAA16B25668269DAF5D8B296FC1F9D06` |
| `…/audit-report.json` | `4FB135F01662163F08FB46869E965EB1F980B6DF2F40C2E5B8F0D70A31338C90` |
| run `manifest.json` | `8B87D437CF769A2B2EF24DBC272E76AAAEC40869C3ECB989C3A2971158594998` |
| run `analysis/gateway-decode-proof.json` | `F306524B597F768E798069856B6B7AB7CC30E9BF7BF37782F922C6BA91DE187D` |
| run `analysis/actual-created-self.bin` | `816FB9EF0DBE1B818E98DE28A968C1E608B051B89C49E925182397BD136FF9A5` |
| run `character-cave-won-held.json` | `2A9F4BA8638A5779F8857C5C399589D6B80FDF6BE7E7D3AA4AC0B5F1B6E51CD7` |
| run `character-final-world.json` | `4C7C035F05274DF1C56864BCCDCC79B09B68945BA4C37A97E3FB296B1567C2CA` |
| release `osfr-gateway.exe` | `4D7226E066D9262A211DD9EBB58F8F0D4310DAB2DA826BCE0E8898CA31B902BC` |
| branch `source-character.json` | `ECEBABE659DAEB380B7803AAD62656A4E418777B756EC13D3ECC1A953A8B0B93` |
| branch `identity.json` | `76C814E8873C4CB136589BDC4A555BCE2F2E5CBAA126806436C375884622DF25` |
| run `native-full-cave-replay.mkv` | `145DE45E5E28AB1D6784518A55DECC2F02B1232F235DC28BFEBDCB3A7F936020` |

Commands: `python audit.py` (exit 0, `AUDIT_OK = True`); raw-frame hashing and CRC via `audit.py`;
snapshot decoding and screenshot inspection via read-only PowerShell/`view_image`. Cross-links:
[[Opening Cave Replay Code Review]], [[Research/Free Realms Recovery]],
[[Design/MMO Worker Contract]].
