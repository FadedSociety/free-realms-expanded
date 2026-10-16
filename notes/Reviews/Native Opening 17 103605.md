# Native Opening 17 — 103605

2026-10-09. **Stages 15-17 and the economy candidate verified on the ended capture. One claim correction: this run contains `39/25` only — no `39/26`/`39/27` packet exists.**

## Capture receipt

`lab/runs/client-baseline-20261009T103605Z/manifest.json` SHA256 `4cba3ab12fbea04390707365f366990a714b6f60bd285de8ee492da57bf9d4b0`: 16135 frames = 18 login + 16117 gateway, 16115 CRC-verified at length 3, complete reassembly (independent analyzer run reproducing the supplied `analyze-native-created-character.py`), self 154005 bytes. `client_stopped/login_stopped/server_stopped true`, client PID 31148 exit `0xffffffff`, elapsed 1973.75 s. `server_executable_sha256 a86615d22657f92571300dfc0b853c089d5f5eabdaace5d996ae2533b4824f3a` equals `lab/state/opening-economy-native-20261009T103555Z/branch.json` `gateway_sha256`, and the manifest identity file matches the branch `identity_sha256`.

## Stage 15 — Smelting 2002 then Forging 2003

Smelting instance `77000002`: client `39/5` frame 1508, four round reports `39/25` frames 1846/3319/4151/4580 (value 8), rounds 101–104 stats `[13,46,26,128] [0,1,0,61] [0,0,0,21] [0,0,0,129]`, final totals time 341 / bad 13 / good 47 / perfect 26 / displayed quantity 2 (21 `FRServer_StatUpdate` fields). `39/18` frame 5920 `won=1, stars=0, elapsed=341, score=5779` matching the held `YOU WIN! Smelting: Copper Bar — TOTAL SCORE 5779` screenshot; `39/47` score row, `39/19` frame 5922, client `39/7` frame 5928, real Close `26/42` frame 5970. Recipe (cleetus15-accepted → smelting-won): ore 2739 instance 32 15→10 (**5 ore**), coal 879 instance 23 15→12 (**3 coal**), bars 21785 instance 24 10→20 (**10 bars**).

Forging instance `139000003`: client `39/5` frame 6639, `26/12` movie `game_forging.gfx` frame 6642, `OnLevelInitMsg 1 180 7`, `OnQuotaInitMsg 0 606 5 1 910`, **55** real `OnChainMsg` chains, five hammer falls (client `OnLevelEventMsg 50` frames 7734/9536/10739/11105/11892) driving quota updates 1..5, `OnStopLevel 1 true` frame 11896, `OnGameWonMsg` 11945, `39/18` frame 11981 `won=1, elapsed=0, score=39300`, `39/47` row, `39/19` 11983, client `39/7` 11985, real Close `26/42` frame 12023. Saved forging run: chains 55, collected 5, score 39300, reshuffles 1. Recipe (smelting-won → forging-won): bars 21785 instance 24 20→10, instances 36/37/38 (defs 2752/55/56 ×5 each) retired, hammer 30280 instance 31 1→2.

Stage 15 acceptance supplies are `38/2` frames 1423–1425 (defs 2752/55/56 as instances 36/37/38, counter 36→39). Cleetus turn-in frames 12058–12061: `38/19` 289 (+10), `38/14` (16,1,50) Blacksmith 25→50, `49/4` quest 910015, `49/12` value 15. `character-cleetus15-turnin.json` and `character-cleetus15-repeat.json` are byte-identical (SHA256 `3d248d925e3ebcf9…`).

## Stage 16 — Queen tour, unlocks and economy

`49/3` offer frame 12144 (quest 910016). Ordered talks at frames 12183 / 12214 / 12239 with objectives 91001601/02/03; `profiles.unlocked` gains 11 and 35 (`[1,11,14,16,35,43,45]`); thirteen `38/2` grants frames 12117–12141 create instance ids **39–51** (defs 18783, 18695, 510, 18882, 18482, 4622, 1238, 11530, 353, 11709, 354, 11828, 4266) and `next_instance_id` moves 39→**52**. Turn-in frames 12322–12325: coins 289→299, Blacksmith XP 50→75, `49/4` quest 910016, `49/12` value 16.

## Stage 17 — classes, hat and final turn-in

`49/3` offer frame 12349 (quest 910017). Five objectives at frames 12683 / 13785 / 14855 / 15411 / 15747 (91001701–91001705) with active profile sequence 16→1→11→35→45 (native Adventurer/Medic/Archer/Chef). The Student Chef Hat is equipped at `equipment[83] = {profile_id 45, slot 1, item_id 16, category 1}`. Turn-in frames 16062–16065: coins 299→309, Chef XP 50→75, `49/4` quest 910017, `49/12` value 17. `character-after.json` shows all 17 stages `accepted/completed/rewarded`, coins 309, Chef active 45, unlocked `[1,11,14,16,35,43,45]`. `character-sunspark17-turnin.json` and `character-sunspark17-repeat.json` are byte-identical (SHA256 `4bde1e42bad5c171…`).

## Visual and binding checks

Viewed directly: `smelting-results.png` (`YOU WIN! Smelting: Copper Bar`, 5779), `forge-batch-56.png` (`YOU WIN! Forging: Spunky Scrapper Hammer`), `cleetus15-complete.png` (Blacksmith HUD, "Forging for Reinforcements" complete, next "Visiting the job Centre"), `class17-hat.png` (Chef class, Student Chef Hat equipped) and `opening17-complete.png` (Captain Sunspark, "Completed: Talk to Captain Sunspark", "Opening story complete"). Movies: `26/12` frames 1511 `game_smelting.gfx` and 6642 `game_forging.gfx`. Release binding: manifest and branch both carry `a86615d2…`.

## Corrections and limits

The claimed `39/25-27` report series is `39/25` only: the 39 sub-op histogram over every forwarded application is `{5:2, 7:2, 16:2, 17:2, 18:2, 19:2, 25:5, 47:2, 14:336}` — no `39/26` or `39/27` exists in this capture (four smelting + one forging `39/25` with values 8 and 3). The run used **9** developer teleports (`type realm goto` rows in `external-inputs.jsonl`), so this receipt proves the stage quest/economy chain, not a walking route or complete historical starting-area restoration.

## Evidence

- `lab/reports/native-opening17-103605-review-20261009T111228Z/verification-proof.json` — `bf10ecf7575346a068888a15798ea01789edbf93f5a74227edb8c881119be553` (all assertions above).
- `.../wire-index.txt` — every op38/39/49/26 message with frame; `.../snapshot-diff.txt` / `snapshot-diffs.json` — the 22-snapshot chain.
- `.../extract_run.py`, `.../diff_snapshots.py`, `.../verify_run.py`, `.../artifact-hashes.json`.
- Run analysis: `lab/runs/client-baseline-20261009T103605Z/analysis/gateway-decode-proof.json` (16117/16115/complete), written by the supplied analyzer; no other run file touched.

Reviewer changed no source, state, shared note, UI or process.

## Restart 111125 — end-of-run persistence gate

`lab/runs/client-baseline-20261009T111125Z/manifest.json` SHA256 `e8e8317bef636251a24e1a260a1cd53855b8e772672ee5c6ed7d2d7e1c6e9576`: 2138 frames = 21 login + 2117 gateway, 2115 CRC-verified at length 3, complete reassembly, self 154395 bytes (`f2dc1454ec6642f8…`), services stopped, client PID 9552 exit `0xffffffff`, elapsed 413.922 s, same release `a86615d2…`, `state_file` = `lab/state/opening17-restart-20261009T111104Z/character.json`.

The branch receipt names `client-baseline-20261009T103605Z/character-after.json` (`source_sha256 4bde1e42bad5c171…`) as the copied checkpoint. The natively reloaded snapshot `character-native-reloaded17.json` has **zero whole-gameplay differences** against that source — only one position/rotation record differs — and preserves `equipment[83] = {profile_id 45, slot 1, item_id 16}` (Student Chef Hat on Chef), `next_instance_id 52`, coins 309, all 17 stages accepted/completed/rewarded, unlocked `[1,11,14,16,35,43,45]` and active 45. `character-restart17-repeat.json` is byte-identical to it (SHA256 `2043178442169135…`, 33503 bytes).

Distinction the checkpoint must keep: the branch directory's `character.json` was rewritten by the gateway at the end of the run and now equals `character-after.json` (`ecebabe659daeb38…`), differing from the reloaded checkpoint only by active profile 45→43 and five movement fields. That is the legitimate post-input ending — the recorded `click 126 174 / click 826 876 / type realm goto Darkthorne` batch at 11:15:45 switched to Brawler and applied a developer teleport — not a restart-load change. The preserved before-input restart checkpoint is the receipted source copy plus `character-native-reloaded17.json`.

No dungeon replay was started. The restart wire contains no `39/x` minigame packet at all, and the server log's progress dump keeps `encounter.active: null` in both the session-start and final saves (`world_stage 0`, `rewarded [411]` unchanged). The only client interactions were the hat menu (input 11:14:50), one Captain Sunspark prompt (`26/9` frame 1456, NPC id 27), the class switch (`26/13` id 43 frame 1987) and the Darkthorne teleport (`15/3` frame 2008). The `native-restart-dungeon.*` filename is only the ffmpeg capture name — `native-restart-dungeon.json` holds video/audio metadata and no dungeon state. `restart-hat-menu.png` was viewed: Chef selected and the Student Chef Hat equipped after the restart. The post-campaign dungeon entry was not found in this run, so this receipt proves the 17-stage economy checkpoint persists across a real client restart; it does not claim a dungeon or a walk route.

- `lab/reports/native-opening17-103605-review-20261009T111228Z/restart-verification.json` — `63adebbfe7cef089d597dcedd7765961cdd0008980bdc0200cb1305858c259a7`, all restart checks pass (private proof only).
