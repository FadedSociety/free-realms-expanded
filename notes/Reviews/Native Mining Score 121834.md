# Native mining score run 121834 independent audit

Read-only audit of `lab/runs/client-baseline-20261009T121834Z` (native cave score replay + Mining18
dev board), 2026-10-09. Reviewer owns this note and
`lab/reports/native-mining-score-121834-review-20261009T1245Z/` only; no source, test, shared note,
state, UI or process was touched and no window was driven. Skills: RTK (normal commands), Ponytail
(reused the previous audit helper by copy, private-parameterized), Caveman (terse report),
game-automation/reverse (raw frame, cave and board evidence read rather than re-run).

Verdict: every claimed number reproduced from the raw frames, snapshots and screenshots. The cave
score row is corrected (`TOTAL SCORE 5000`), the Mining board ran natively on `18000001` with one
accepted drag (`0→1` chain, `0→400` score) and a clean quit, and the two open render issues
(blue regulars, both quota rows labelled Iron) are real and **not** claimed fixed; no full-Mining
win, accepted row or reward claim exists in the run.

## Raw evidence re-derived

`audit.py` (copy of the accepted cave-replay helper, re-pointed and re-parameterized for this run)
prints `AUDIT_OK = True` (`audit.log`, `audit-report.json`):

| Claim | Re-derived result |
| --- | --- |
| Frame ledger | manifest 7,440 frames = **7,415 gateway + 25 login**; every frame's length and SHA-256 matches (0 mismatches), no missing or extra `.bin` |
| CRC / reassembly | handshake `crc_length 3`; my own CRC32 low-3-byte check passes on **7,413 / 7,413** non-handshake gateway frames with exact decompression and opcode agreement; reassembly complete (client 2,527 messages/5 duplicates, server 363) |
| Applications / self | **3,077** application messages; one created-self, 154,395 bytes, SHA-256 `1589AD7A…`; all eight values byte-equal to `analysis/gateway-decode-proof.json` |
| Release | `prototype/osfr-tools/target/release/osfr-gateway.exe` SHA-256 `6935FD25…` = manifest = `branch.json` |
| Branch | `source-character.json` = `111…114509/character-after.json` = `4C7C035F…`; identity `76C814E8…`; state dir adds only the mining block (final `character.json` = `mining-quit-settled`) |
| Optional package | `opening-mining-native-20261009T121330Z` = 36 files byte-identical to `opening-anvil-placement-v1` **plus** `content/mining.json` (`25370F04…`) and `mining-receipt.json` (`8D025E62…`, documents "no configured rewards") |

## Cave score sequence (instance 5)

Server `gameplay_saved` order: `id=5/loading hp=1000` → fighting (hp 1000→900→700) → `id=5/won
hp=700` → `active=null next=6 hp=1000` (real Close → world). `character-cave-full-health-entry`
(id 5, hp 1000), `character-score-fight-stopped`/`-won-held` (won, hp 700),
`character-score-world-return` (active None, hp 1000). Fight helper logs show the full enemy set
(150/150/150/500) and two bounded batches ending `"dungeon defeated"`, kills 4, hp 700.
`score-corrected-win.png` reads **YOU WIN!**, `Opponents Defeated 4 – 0`, `Goals 1/1 – 5000`,
`TOTAL SCORE 5000` (the previous run rendered `0`), so the corrected score row is verified visually.
Inventory/profiles/opening are unchanged in every snapshot (coins 309, counter 52, 17 rewarded
stages 910001–910017).

## Mining 18 sequence (board 18000001)

States from `character-mining-*.json` and the server event stream:
`Offered → Launched → Connected → LevelOffered → Playing` (instance `18000001`, `remaining_ms
180000`, paused at offer) → `Playing` unpaused (`rem 100420`) → paused via the UI pause button
(`click 860 26`) → the accepted drag → `Playing score=400 chains=1` → paused again →
`active=null` (quit settled). `native-mining-first-chain-proof.json` records the mapping exactly:
server coords `[[4,7],[4,8],[4,9]]` = physical `[[526,530],[526,574],[526,618]]`, `chain_delta 1`,
`score_delta 400`, `paused_before_after [true, true]`, inventory/opening/profiles unchanged, and the
explicit `no_reward_claim`: "One normal native drag accepted. Full Mining win/rendering/restart not
accepted." `external-inputs.jsonl` contains the matching `mdown 526 530` / `move …574` / `move …618`
/ `mup` commands and the quit `click 420 26`.

Quit path: `character-mining-quit` (still `Playing`, score 400, paused) → `character-mining-quit-settled`
(`active null`) → `character-mining-world-return`; the repeated quit produced the **TRY AGAIN**
transient overlay (`mining-quit-second.png`) that auto-closed into the world. Final state:
`mining {next_instance_id 18000002, completed 0, active null}`, coins 309, counter 52, stages 17 —
economy and progression unchanged. `mining-board.png` confirms the open render issues: regulars all
render blue and both quota rows read "Iron" (0/15 and 0/6); `content/mining.json` is the
reconstructed 14×14 board spec (`reconstructed_rules: true`, game 18, profile 14, collectables
601/602).

Recording/stop: `native-mining-score.stop` present; `.mkv` 108,430,565 bytes; audio 317,020,784 bytes
(= 825.6 s at 48 kHz stereo f32); ffmpeg log 0; manifest `client_stopped/login_stopped/server_stopped`
all true with client pid **16636**; `launcher.log` ends `Captured datagrams: 7440`.

## Limitations and non-claims

* No full-Mining completion, no accepted result row and no reward are claimed anywhere; the two
  render issues stay open, as does the native keyboard binding for the board.
* The run used the external input helper (physical clicks/moves/holds); this is real native input,
  not a purely manual playthrough.
* Recording validity rests on the stop flag, EBML presence and non-zero track sizes; no ffprobe
  duration decode was available.
* `mining.json` rules (board, quotas, spawn rates, seed, outputs) are reconstructed, not recovered;
  the envelope/shape/chain/queue math is described as recovered from `game_mining.gfx` in the spec
  notice itself and is not re-verified here.
* Single run; the cave score correction is demonstrated once, with the prior run as the `0` baseline.

## Evidence

| Artifact | SHA-256 |
| --- | --- |
| run `manifest.json` | `4167ECCB8B7A11CC841483C8A8D2E6D7178A08DF0B912B849CD1B5D6FFE85CDF` |
| run `analysis/gateway-decode-proof.json` | `264DDF62459B32C8860F6043CE4DAE9B8B0D67ECE8C5D7247DEEFC63EECF7DA1` |
| run `analysis/actual-created-self.bin` | `1589AD7A917E915AACB68AED9EFEFDD5A9CC8FDE2BEDE19111E9F728D881FDA0` |
| run `character-score-won-held.json` | `0D751BB5AF96F4C9589317218645B82CE96537D8A8CA41369FE7C83ACD9F20D4` |
| run `character-mining-after-chain.json` | `93B3BE50D54EF0C1E1487148AD4A0330EF56EA1A25CB403FBD0D4812AB9DB865` |
| run `character-after.json` = quit-settled | `A6ADF50820DA57D102CBAA71AD18826D4177AF3427E687EA72281E32AD83AEE0` |
| run `native-mining-first-chain-proof.json` | `E582156EDC5E12FD48AF1BFF56E9DA1263E0EA525FCF7FC75110F9D7827B35A7` |
| release `osfr-gateway.exe` | `6935FD259D6380224267966355E70E12750112A9641907323A6F9B0CB58F3E8C` |
| package `content/mining.json` | `25370F048EC87F12C30B38AC338D1585DE3C04BBD2C080185981A1D4C8E33C8B` |
| package `mining-receipt.json` | `8D025E6214315393BBED64D4A2DAA2F63D0E8983C381195AC1C8BFB4D209FA61` |
| run `mining-board.png` | `96466AAC1D3EA94B860D252D6A5CE2D5A5B0368C6B08934DCCB4067B07749BFC` |
| run `score-corrected-win.png` | `DA8A4A80820FF70B5D2A1B766F115079DFDA5D80FBD724D617FAB48BEA678FC3` |
| run `mining-quit-second.png` | `DFA4A241581AB2DE0BCBD04E4BAFA1951153258FFCD08AA593FDB19CD55FFF1A` |
| `native-mining-score.stop` | `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` (empty stop flag) |
| private `audit.py` / `audit.log` / `audit-report.json` | `7ED59342BF77208C…` / `C4135149F7049AA0…` / `E91F9809C2CB20AC…` |

Commands: `python audit.py` (`AUDIT_OK = True`); frame hashing, CRC32-3 and reassembly are the
helper's own code, re-pointed to this run; snapshots and screenshots inspected read-only. Cross-links:
[[Native Cave Replay 114509]], [[Opening Cave Replay Code Review]], [[Design/MMO Worker Contract]].
