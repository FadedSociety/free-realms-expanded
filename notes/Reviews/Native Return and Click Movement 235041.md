# Native Return and Click Movement - 235041

October 8, 2026. Bounded, read-only independent audit of the finalized native run `client-baseline-20261008T235041Z`. 34 automated checks passed with no failures (`../../../lab/reports/native-return-click-235041-20261009T0038Z/audit.json`, SHA-256 `c3ca9a468d12a06563cc280043e3298e5331ca0ca813b390bccf15450099ef85`). No UI, native launch, source/save/content/client edit, run-dir write, export or commit was made by this review.

## Finalization and capture integrity

Release `ea297301afc5462be7f40142d2a6b4709eb94af6b3556de721dfa407d9a89063`; 6699 manifest frames (6682 gateway + 17 login) over 2433.656 s; manifest reports client, login and server all stopped. The recorded `client_exit_code` is 4294967295 (unsigned -1); the harness separately reported exit 0, so this note keeps both facts rather than treating the unsigned code as a failure.

The live capture prefix equals the final manifest exactly (6699/6699 entries), `frames/live-status.json` is `closed`, `authoritative:false`, `written_frames:6699`, `pending_frames:0`, no error or incomplete flag, and every one of the 6699 frame files matches its manifest byte length and SHA-256. CRC and reassembly were re-derived read-only with the shared `analyze-native-created-character.py` algorithm (that tool writes into the run's `analysis/`, which this audit deliberately did not create): 6680 of 6682 gateway frames verified, 2 session frames (opcode 1/2) skipped as the shared decoder does, 0 CRC/opcode errors, complete reassembly (client-to-server 2649 messages including 55 duplicate sequence replays; server-to-client 55 messages; no queued or partial fragments).

## Whole-state preservation

Canonical JSON with `character.position`/`rotation` normalised:

| Pair | Gameplay identical | Evidence |
| --- | --- | --- |
| Queen source vs run `character-before.json` | yes | both SHA-256 `3a9d38078449b7dd95fdd8263a733b9d77e44b24ffe740d1e0a44deff07e2681` |
| `character-before.json` vs `character-after.json` | yes | only position/rotation change: `-1063.606,4.4188,305.6823` to `-1999.9143,-42.6559,445.8581` |
| `clickmove-before.json` vs `clickmove-after.json` | yes | 259 coins, 30 items, Brawler 43, stages 1-12 rewarded, stage 13 still unaccepted |

`identity-before.json` and `identity-after.json` are byte-identical (`76c814e8...`). `character-after.json` is byte-identical to `character-before-developer-restart.json` (`44e5f207...`), so the marker file is a copy of the finalized save, not a second state.

## Return walk from the Queen

Route candidate `../../../lab/reports/queen-mae-road-route-v3.json` (44 waypoints, 1316.78 m, `native_route_accepted=false`). 31 navigation logs show arrivals at indices `1-15`, `17-24`; the start index 0 is the Queen-side checkpoint (9.43 m from waypoint 0).

- Bridge gates: the route documents the two original `sg_lilypad_bridge_01/02` spans over the Sanctuary pond at indices `[0,1,2,4,5]`; every gate waypoint except the start itself was reached with ordinary movement.
- Ramp/downhill: climb-gate indices 20-26 (about a 28 degree upper-road section) were entered and traversed through 24; 25/26 were not reached.
- Guard and timeout events retained: index 7 and 17 "two ineffective batches" guard stops followed by fresh same-target arrivals; index 16 "navigation time limit reached" with no retry (walk continued at 17); index 21 timeout followed by a fresh arrival; **index 24 shows the wrapper timeout** (`navigation-20261009T001524729358Z.jsonl` has neither arrival nor a stopped event) and a later fresh same-target arrival 1.97 m short (`navigation-20261009T001641984749Z.jsonl`).
- Index 25 was never delivered: `recorded WinDrive failed ... error target window is not in the foreground (foreground: ChatGPT)`, the same foreground guard that blocked record 378 before the click experiment.
- No Mae arrival: furthest arrived index 24 of 43. The server trail holds 1124 saved positions with an estimated 1040.4 m planar path.

Retained stills sampled for this review: `queen-return-bridge761.png` shows the lilypad crossing ahead; `queen-return-downhill.png` shows the cobbled descent road; `queen-return-bridge803.png` and `queen-return-first-bridge.png` show the Sanctuary-side route terrain. The full recording `queen-return-playthrough.mkv` (639738449 bytes) plus its raw audio and stop note are retained unmodified; only first/last 4 MiB hashes were computed and no export/transcode was performed.

## Click-to-move experiment

`config-before/UserOptions.ini` has `ClickToMove=1` and `ClickToMoveRightButton=0`. Input record 379 (17:19:32 local) is exactly `focus | fg | click 676 618`, delivered with FreeRealms confirmed foreground. Between `clickmove-before.json` (17:15:46) and `clickmove-after.json` (17:19:50) no movement key was delivered: the only movement attempt (record 378, 17:17:14) was refused by the foreground guard with ChatGPT in front and never reached the client.

The character moved 6.9444788798 m planar (6.9894456847 m 3D), matching `clickmove-test.json` exactly, while the whole `gameplay` object stayed identical. The server save trail between the snapshots contains a duplicate start save (float jitter only) then two steps of 3.404 m and 3.585 m; both steps align with the overall heading (dot 0.99997 and 0.99998) and the 6.9896 m path matches the 6.9894 m straight line. That is the shape of ordinary click-to-move walking, not a single jump.

## What this proves and what it does not

Proves: the finalized capture is hash/CRC consistent and completely reassembles; the run changed no gameplay state or identity; the Queen source checkpoint is preserved byte-for-byte; an ordinary foreground click with click-to-move enabled moved the character about 6.94 m while no movement keys were delivered; the return walk used the documented bridge-gate and ramp waypoints with retained guard/timeout handling, and stopped before Mae. The lead reports no save coordinate edit or teleport in this old-release run; the input log, continuous save trail and unchanged release are consistent with that.

Does not prove: packet-level client movement decoding, bridge/ramp collision or support (the route file itself is still a candidate and says the reverse bridge/ramp require original-support observation), acceptance of the whole 44-point route (index 16 skipped by its time bound, 25+ not reached), any Mae arrival or stage 13 acceptance, or that the click target corresponded to a specific world point. The dev-teleport work planned for a future release is separate and is not exercised or evaluated here.

Evidence: `../../../lab/reports/native-return-click-235041-20261009T0038Z/` (`audit.json`, `receipt.md`, `audit-native-return-click.py`). Key inputs: run `manifest.json` `95cddbfe...`, `frames/live-index.jsonl` `2626800c...`, `external-inputs.jsonl` `02080749...`, `server-stdout.log` `4f777c25...`, `clickmove-test.json` `598343f5...`, route `adcf9911...`, point-24 logs `b884dbf7...` and `45dffb5a...`. See [[../Current Progress]].
