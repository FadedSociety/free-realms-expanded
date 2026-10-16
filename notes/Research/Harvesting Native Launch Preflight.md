# Harvesting Native Launch Preflight

Read-only audit for opting in a native Harvesting 111 launch under Chef45. No production, test, content, save, UI or process change was made. Reviewed revisions: `src/harvesting.rs` `4e5196ba35ab99c6`, `src/mining.rs` `e6e421d8e7cd0e92`, `src/gameplay.rs` `077ef90684d026f5` (lead-owned, in flux), content `opening.json` `3e6f4228efc76460`, `cooking.json` `60d208f566699124`, activity catalogue `f02d21a7104f075b`, job binding receipt `5ff91b32fff7bdf9`. Evidence bundle: `../lab/reports/harvesting-native-preflight-20261009T133000Z/`.

Verdict: Chef45 is the supported job gate for a harvesting launch used by cooking, and the launch wire is the already-proven shared MatchThree family. The one thing that cannot be closed from recovered data is ingredient conservation: recovered row 111 spawns stickfruit/stainberry/tumtumnip/spiralmint, not the StrongArm Stew bundles gummyfruit 2875 / rainleaf 2884 that the local farmstead gather path currently grants. Do not replace that gather path and do not invent a Farmer or profile 111.

## Recovered versus reconstructed

Recovered (with source):

| Field | Value | Source |
| --- | --- | --- |
| Harvesting game ids | 58, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 217 | MiniGameData, TYPE 8 |
| Region 111 identity | name `100834`, description `94150`, icon `908`, group 8, Flash client mode | MiniGameData + `MiniGameTypeData.IS_FLASH`; catalogue `f02d…` |
| Region 111 produce order | stickfruit, stainberry, tumtumnip, spiralmint (locale text) | locale `94150` |
| Bundle items | 2885/2887/2890/2877 for 111; 2875 gummyfruit, 2884 rainleaf | pinned `ClientItemDefinitions.json` |
| Movie / preload | `game_farming.gfx`, `FarmingAssets.txt` | `AssetsW_009.pack` 1448 / `Assets_000.pack` 50 |
| Launch wire scripts | `GameClientNetwork.as` `571592993e9b9d72`, `GameServerNetwork.as` `8c2b5c5a90b3cdc4`, `GameData.as` `fc779888deb39e9d` — byte-identical to Mining18 | decompiled movies |
| Farmstead gather nodes | `...5068` rainleaf (resource 1001), `...5069` gummyfruit (1002), `required_profile: 45` | `opening.json` |
| Chef job relationship | Chef XP comes from the harvesting and cooking minigames | wiki Chef rev 46136 `870b47c6…` |
| Farm produce is not cooking produce | "Vegetables or fruits that are harvested from a farm will not work in cooking, only from the harvesting mini-game." | wiki Harvesting rev 48154 `35ef6668…` |
| Farmer is separate | Farmer = housing-farm progression, outside the job rotation | wiki Farmer rev 45266 `69ff7be9…` |

Reconstructed or unresolved (explicit):

| Field | Current state |
| --- | --- |
| `profile_id` | `harvesting.rs` sets `profile_id: game_id` (`:921`) with a placeholder comment — not a usable gate |
| Board geometry/quota | 10x9 rectangle + dropzone row, quota 15, 5 tiles per bundle, spawn percents — labelled reconstruction |
| `seed`, instance namespace | `seed = game_id`, `58_000_001` — local choices |
| 39/16 capture of the original launch | locally authored: native handler shape plus three **empty** 69-byte reward-bundle records (`mining.rs:753-772`); the original bundle/preview payload is unrecovered |
| 26/12 `MiniGameFlash` launcher | locally authored; the pinned community set covers 39/17, 39/18, 39/19, 39/47 but has **no** launcher or 39/16 writer |
| MiniGameData 111 spawn set | locale says four produce; the wiki Chef page's "Farnum's Farm" harvesting list says six (adds gummyfruit + rainleaf). Which set the original minigame spawned is unresolved |
| World entry | no recovered door/NPC/activity trigger for 111; `piece_018` marks it `Entry::ModuleDriver` |
| `ConfigServer.GAME_ID = 60` | stale local test default (MiniGameData 60 = Mail Sorting), not evidence |

## Exact minimal changes to opt in 111/Chef45 (proposal only)

1. `src/harvesting.rs`: replace the `profile_id: game_id` placeholder with an evidence-backed launch profile. Smallest shape: a region-bound `profile_id: i32` (45 for the cooking-related regions) carried into `HarvestContent`, with `validate_content` refusing any profile that is not 45 (explicitly refusing `111`) and `tests/harvesting.rs` asserting `content(111).profile_id == 45`.
2. Shared engine: no change. `Type8Engine::load`/`begin(profile)` already compares the caller profile and embeds it in the `39/16` info packet (`mining.rs:756`), and `Type8BoardSpec` already supports up to 8 goals (`MAX_GOALS = 8`, `mining.rs:85`) if a six-produce variant is ever accepted.
3. Gateway dispatch (lead-owned): add harvesting only behind the existing dev gate, `begin(45)` with content `harvesting.json` for game 111, instance namespace `58_000_001`; wrong active profile must be refused before any packet is sent.
4. Content: write `harvesting.json` for 111 with `harvesting::write_content(dir, 111)` into the run content directory; it already advertises `FarmingAssets.txt` and `game_farming.gfx`.
5. Ingredient conservation: **do not** wire the run's outputs into the stew recipe and **do not** remove the gather path (`gameplay.rs:1216-1223` grants 2884/2875 per newly counted Chef interaction). Either the launch is the recovered-4 activity (no stew coupling) or the lead accepts a labelled six-produce reconstruction (locale four + wiki gummyfruit/rainleaf) whose bundles may later feed the stew, with a conservation test.

## Test-first acceptance cases

1. `harvesting::content(111)` reports `profile_id == 45`; `validate_content` rejects a content with `profile_id == 111` and with a Farmer placeholder.
2. Real engine load + `begin(45)` for 111 reaches Playing; `begin(111)` and `begin(14)` fail with the profile mismatch.
3. `39/16` carries 111's name `100834`, description `94150`, icon `908`, `FarmingAssets.txt`, and profile `45`; `26/12` requests `game_farming.gfx` and never `game_mining.gfx`.
4. Conservation: the 111 run's outputs are exactly 2885/2887/2890/2877 (no 2875/2884), the gather path still grants 2884/2875 once per newly counted interaction under Chef45, and a replayed run grants nothing.
5. If the lead accepts the six-produce variant: `board_spec(111)` has six goals/outputs in the documented order, `validate_content` labels it reconstructed, and the same conservation case proves the stew materials can only arrive once per source.
6. Gateway gate: a Chef45 character launches; another job's interaction is refused with no `39/16`, no `26/12` and no state change.

## Unresolved launch evidence

1. No recovered world entry/trigger for Harvesting111 (activity group 8 is a catalogue, not a trigger).
2. The original 39/16 reward-bundle/preview payload (3 × 69 bytes) and the 26/12 launcher are not written by any pinned community writer; both are locally authored.
3. Locale `94150` (four produce) versus wiki Chef page (six, adding gummyfruit/rainleaf) — the minigame's historical spawn set is unrecovered because the `MatchThree*Data.txt` server tables were never shipped.
4. No capture of a harvesting `39/5`, `39/14`, `39/17`, `39/19` session exists.
5. The original score policy for harvesting shares the unresolved cave score question (see [[Cave Results Score Binding]]).

## Hash check

`hash-check.log` re-verifies every hash and line cited above; failing checks are printed as `FAIL` and none are expected.

## Implemented job binding (2026-10-09, authorized)

The minimal binding repair is applied; harvesting stays an unregistered separate candidate.

- `src/harvesting.rs` `272711da0e6b72fc…`: new `pub const CHEF_PROFILE_ID: i32 = 45;`; `content()` now writes `profile_id: CHEF_PROFILE_ID` instead of the game-id placeholder, and `validate_content` refuses any other profile with an explicit message. Schema, board geometry, quotas, produce sets, output bundles, instance namespace and launch assets are untouched, and `src/mining.rs` stays frozen at `e6e421d8e7cd0e92…`.
- New `tests/harvesting_job_binding.rs` `20ae461056c65090…`: all six supported regional specs report `profile_id == 45`, `profile_id != game_id`; `validate_content` refuses the old `111`-as-profile shape and a Farmer-like `15`, accepts Chef; gap profiles still refuse launch.
- In-repo RED (`repo-red-harvesting_job_binding.log`): 0 passed / 2 failed — game 109 bound profile 109 and a wrong profile validated. In-repo GREEN (`repo-green-focused.log`): `harvesting` 17 passed, `harvesting_job_binding` 3 passed, `harvesting_review` 4 passed, 0 failed.
- Two necessary one-line expectation updates outside the original ownership were authorized and applied: `tests/harvesting.rs` `b01020e483377cc8…` now asserts `harvesting::CHEF_PROFILE_ID`, and `tests/harvesting_review.rs` `80ca4bc4e91ce6cc…` begins the run with `harvesting::CHEF_PROFILE_ID`.
- `rustfmt --edition 2024` on the four touched files exits 0; it also normalised pre-existing formatting drift in those files (whitespace only).
- Unchanged and still open: world entry, 39/16 reward bundles and 26/12 launcher authorship, the locale-vs-wiki spawn-set question, ingredient conservation policy and the quest gather path (`gameplay.rs:1216-1223`), which received no edit (`gameplay.rs` hash unchanged).

Private bundle: `../lab/reports/harvesting-job-binding-fix-20261009T140000Z/` (`repo-red-harvesting_job_binding.log`, `repo-green-focused.log`, `final-evidence.json`, plus the earlier scratch RED/GREEN preparation logs).
