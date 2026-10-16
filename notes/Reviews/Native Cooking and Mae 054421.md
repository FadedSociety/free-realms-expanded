# Native Cooking and Mae — 054421

2026-10-08 (capture 2026-10-09T05:44:26Z–06:06:14Z). **Cooking stage13 with Mae and Miner stage14 with Cleetus accepted on the complete final capture. No stage15 is accepted.**

Reviewer read only: run artifacts, the shared writer source for packet-field semantics, and the pinned independent results vector. No UI, process, save, shared code or shared analysis directory was touched; every produced file lives in the private report below or in this note.

## Capture receipt (final manifest)

`lab/runs/client-baseline-20261009T054421Z/manifest.json`, SHA256 `9b3a81c06f7e11821c6f0273de09801b6eac19e1c1573df7db7dd3648c828f14`. Frames 9206 = 18 login + 9188 gateway; every frame size/SHA256 verified, 9186 CRC-verified at length 3, complete reassembly on both directions. `started_utc 2026-10-09T05:44:26.690539+00:00`, `elapsed_seconds 1305.969`, `server_kind rust_gateway`, `client_pid 21188`, `client_exit_code 4294967295`, `client_stopped/login_stopped/server_stopped true`, `asset_delivery_proofs []`. The live-prefix writer closed at 9206/9206 with no pending, incomplete or error state, no unreferenced frame on disk, and every frozen private copy equals the manifest frame set byte-for-byte.

The frame `time` clock was anchored through the harness image mtimes: five anchor pairs agree to 0.063 s and place the capture origin within 0.03 s of the manifest `started_utc`. Click-to-packet delays: results Close 0.558 s, Mae turn-in 0.552 s, Cleetus turn-in 1.414 s.

## Stage 13 — original Cooking game and Mae

Cooking instance `46000006`, game 46, `CookingAssets.txt` info packet, sub5/sub7/sub16/sub17/sub18/sub19/sub47 all exactly once. `Won 5199`, stars 0, elapsed 198 s; the terminal 39/18, 39/47 and 39/19 equal the pinned independent template with only instance/elapsed/score substituted. Six rounds in order 1,2,3,6,17,7:

| Round | Bad | Good | Perfect | Seconds |
| --- | ---: | ---: | ---: | ---: |
| 1 | 11 | 8 | 0 | 39 |
| 2 | 0 | 15 | 0 | 22 |
| 3 | 1 | 2 | 0 | 23 |
| 6 | 0 | 1 | 0 | 46 |
| 17 | 0 | 3 | 0 | 48 |
| 7 | 0 | 0 | 10 | 17 |

28 `FRServer_StatUpdate` fields (6 rounds × 4 plus totals 198/12/29/10) match the won snapshot exactly; per-round time sum 195 allows the same 3 s transition difference seen before. Six client `39/25` reports `[instance,6,-1]` fire once per round.

Actual Close: the single client `26/42` frame 6413 (`1a002a00`) follows the recorded `click 1140 76`. It clears only `gameplay.cooking.active`; coins stay 259.

Recipe attribution: the four server `38/4` consumes, frames 6368–6371, instances 32/35/34/33 = definitions 2862/2900/2884/2875, each stack of 4 removed entirely, and the single `38/3` frame 6372 that updates existing instance 15 (definition 35) to count 2 (**food 35 +1**) all belong to the `GameWon` resolution burst, i.e. before the results page and before Close; coins stay 259. The `character-cooking-won.json` snapshot taken at that same boundary already shows food 35 = 2, the four ingredient instances gone and coins 259, and `character-cooking-closed.json` is unchanged apart from `cooking.active`. The later Mae turn-in (frames 6448/6452, after Close) grants only `38/19` 269 (+10 coins), `38/14` (45,1,50) (**Chef 25→50**), `49/4` quest 910013 complete and `49/12` stage 13 — no material packet is emitted there. A second Mae interaction (6515) and the later Cleetus offer (6558) add no reward packets; the whole Mae-repeat quiet window to frame 8668 is grant-free.

## Stage 14 — Miner selection, ten ore menus, Cleetus

Miner class selection completes stage 910014 objective 1 (`49/9` frame 8671, objective 91001401, value 0.5, then `49/10`), sets the active profile to 14 and stage 910014 accepted with counts `[1,0]`. Ten valid Copper Ore menus — interact frames 8921, 8942, 8955, 8968, 8979, 8990, 9001, 9012, 9025, 9038, each with the server `"Copper Ore"` prompt — grant one ore and one coal each: instance 32 definition 2739 created by `38/2` (frame 8924) then updated to counts 2..10 (`38/3`), instance 23 definition 879 updated 3..12. Final stacks: **ore 2739 1→10, coal 879 2→12**, matching the 06:03:23 snapshot, whose stage counts become `[1,10]`. An eleventh ore interaction (frame 9112) produces only the prompt and no inventory or quest progress.

Cleetus turn-in (interact frame 9133 after the `"Cleetus Farnum"` prompt): `38/19` **coins 269→279**, `38/14` (14,1,50) **Miner 25→50**, `49/4` quest 910014 complete, `49/12` stage 14. The only other stage-14 wire event is the early `49/3` quest offer at frame 6561 (after the pre-selection Cleetus interact 6558), which grants nothing. Stage 910015 stays `accepted=false` with zero counts in every snapshot; the only stage flags that ever change are 910013 and 910014.

Repeat snapshots `character-mae13-repeat.json` and `character-ore11-repeat.json` are byte-identical to their base snapshots (ore pair SHA256 `046821f9261e9d33b5accc0948782b3bbbec93dad855fc647e0cebe0e3d94312`), and both repeat files carry their base snapshot's mtime, i.e. they record the same checkpoint rather than a post-repeat capture; the no-second-reward claim rests on the wire quiet windows.

## Visuals

`mae13-complete.png` SHA256 `8f1c3baf687a087c…`: original Quest Completed banner, Chef level 1 HUD, "Second Helping of Strongarm Stew Quest Completed!", and the next prompt "Next: Cleetus Copper Collection. Talk to its guide." with the Mae Farnum interact prompt. `mae-turnin.png` SHA256 `d4f5fbcd52c419eb…`: native world with Mae `Press X` prompt and the goal update/complete `1/1` chat lines. Stage14 visuals exist and were viewed: `miner-selected.png` (SHA256 `a62aff3a0ffdd810…`) shows the class/gear panel with Miner selected and the Student Miner Hat slot; `ore10.png` (SHA256 `da472e23c8318442…`) shows the Miner HUD, "Cleetus' Copper Collection / Mine Copper Ore - 8/10" banner, the Copper Ore prompt and the chat log of ore progress 6/10 through 10/10 with "Goal Complete: Mine Copper Ore"; `cleetus14-complete.png` (SHA256 `adbb57740055fcc9…`) shows the Cleetus Farnum prompt, "Completed: Cleetus Copper Collection." and the next prompt "Next: Forging for Reinforcements. Talk to its guide." — stage 15 is offered, not accepted. Each also has a byte-hashed `_small` variant (`bd2595cab60117e3…`, `4bdb0d7a2c401cc1…`, `470b556aae983ae4…`).

## Economy deltas at each boundary

| Boundary | Coins | XP | Items / stages |
| --- | --- | --- | --- |
| Cooking win (GameWon burst 6368–6372) | 259 → 259 | Chef 25 → 25 | 4 ingredient stacks removed; food 35 1→2; stage 910013 count 3 0→1 |
| Close | 259 → 259 | unchanged | only `cooking.active → null` |
| Mae turn-in (frames 6448–6459, after Close) | 259 → 269 | Chef 25 → 50 | stage 910013 completed/rewarded; no material packet |
| Miner selection | 269 → 269 | Miner 25 → 25 | profile 14 active; stage 910014 accepted, counts [1,0] |
| Ten ore menus | 269 → 269 | unchanged | ore 2739 1→10; coal 879 2→12; counts [1,10] |
| Cleetus turn-in | 269 → 279 | Miner 25 → 50 | stage 910014 completed/rewarded; stage 910015 still not accepted |

Every other path in the whole character snapshot is unchanged except recorded position/rotation.

## Limits and unknowns

The six client `39/25` reports recur per round with payload `[instance,6,-1]`; no recovered name or semantic was established here. `38/3` field semantics come from `src/inventory.rs::item_update`; the snapshots confirm the instance/count values used. The live status file stays `authoritative=false` because it is the prefix writer — the manifest is the harness receipt, and the two agree on all 9206 frames. No item-definition byte-equality to the original serializer is claimed, and no claim beyond stages 13/14 is made.

## Evidence

- `lab/reports/native-cooking-mae-054421-review/verification-proof.json` — 6c9708e454064153046f6d768bd60b07c4862421f6d4ee11f0af9784172d7c81
- `lab/reports/native-cooking-mae-054421-review/receipt.json` — a04096df5f8716a4c1b9d67e44487777a59cb2bb6822377927f8703f6bcfda0c
- Superseded versions kept explicitly: `previous/verification-proof-49df7471be976b2f.json` (`49df7471be976b2f04590d338fef65f7c43c4980c9ccaba88b416fb2177da35e`) and `previous/receipt-7616ae7bb4cd3b1d.json` (`7616ae7bb4cd3b1d1e642687b7020d20d6b53568b85f625733a40ec3c7ec3b0c`), which omitted the recipe/turn-in attribution and the stage14 screenshots.
- `lab/reports/native-cooking-mae-054421-review/frozen-decode-proof.json` — decode/CRC/reassembly over the frozen 9188 gateway frames
- `lab/reports/native-cooking-mae-054421-review/frozen-index.jsonl` — 77999ccbaad385f08fb96abf9f32b1c883e507210fd26d5923055f2fb52a0de9 (9206 records)
- `lab/reports/native-cooking-mae-054421-review/freeze.json`, `verify_native_cooking_mae_054421.py`, `analyze_frozen_prefix.py`, `receipt.json`, `artifact-hashes.json`
- Pinned template `lab/reports/native-crafting-results-vectors-independent.json`; shared writers `src/inventory.rs`, `src/profiles.rs`

Reviewer changed no shared Rust, gameplay save, movie, release binary or native input.
