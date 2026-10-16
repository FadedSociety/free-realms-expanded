---
type: research
updated: 2026-10-13
---

# Module Registration (W30)

W30 registered thirteen finished Rust modules in `prototype/osfr-tools/src/lib.rs` and changed the corresponding integration tests to import the public crate modules. No gateway or gameplay routing was added. This exposes the module APIs and exercises shared crate types; it does not claim native integration or feature completeness.

## Registered modules

Requested: `abilities`, `content`, `minigame`, `players`, `social_chat`, `world`, `quest_engine`, `social_lists`, and `mounts`. Additional finished modules found through standalone `#[path]` tests: `derby`, `fishing_service`, `inventory_v2_fixes`, and `soccer_service`.

`startup_selfdata`, `housing_service`, `pets`, and `trade` remain unregistered as directed. During the final suite run a separate `tests/trade.rs` target appeared from concurrent work; W30 did not edit it or register `trade`.

## Test and type alignment

Converted `tests/abilities.rs`, `content.rs`, `content_dungeons.rs`, `content_jobs.rs`, `content_npcs.rs`, `derby.rs`, `fishing_service.rs`, `inventory_v2_fixes.rs`, `minigame.rs`, `mounts.rs`, `players.rs`, `quest_engine.rs`, `soccer_service.rs`, `social_chat.rs`, `social_lists.rs`, and `world.rs` to crate imports. The old `#[path]` trees could compile duplicate module copies and make test types differ from library types.

- Content, world, and quest-engine tests now share `content::ContentDb`, `Quest`, and content records. `world` already consumes `crate::content` and converts the richer content NPC record into packet-facing `startup_world::NpcSpawn`; those records retain distinct shapes for their distinct roles.
- Quest-engine tests share the library `content`, `rewards`, `inventory`, `profiles`, `opening`, and startup modules. Content grants retain provenance; the quest engine explicitly adapts them to runtime reward types.
- Minigame and mounts tests share the registered inventory/profile/reward types. Social tests use the registered chat/list modules and startup-world rows. Fishing and soccer service tests use the registered service and codec modules.

No production type declarations needed edits: the modules already refer to the intended crate types. Alignment is in test imports, which removes the duplicate compilation paths.

## Original-client evidence boundary

W30 recovered no new client facts and changed no protocol behavior. Existing evidence remains in the module notes: startup NPC wire fields from `PlayerUpdatePacketAddNpc_ReadBody` at `00921a10` (`Startup World Content.md`), the content crosswalk (`Content Data Model.md`), world instance boundaries (`World and Instances.md`), and quest/content/reward adaptation (`Quest Engine.md`). The other modules retain their evidence and reconstructed-policy limits in `Player Visibility.md`, `Chat Core.md`, `Ability System.md`, `Minigame Lifecycle.md`, `Mounts Service.md`, `Derby Restoration.md`, `Fishing Service Integration.md`, `Soccer Service Integration.md`, and `Inventory Search Crash.md`. No new reverse-engineering or game/asset workflow was needed for crate registration.

## Verification

Working directory: `project/prototype/osfr-tools`.

```powershell
$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W30'
& ..\..\scripts\cargo.ps1 test --no-fail-fast
```

Final result: exit 101; 199 targets, 1,882 passed, 2 failed, 89 ignored. The two failures are outside W30's implementation ownership:

- `tests/content_jobs.rs:62` (`content_jobs`, 0 passed / 1 failed): W22's assertion expects every job/quest NPC reference to remain in `npcs_unresolved.json`. W27's regional NPC batch resolved 277 keys, leaving 188 unresolved. `notes/Research/Content Batch NPCs.md` identifies the stale assertion and says that test is outside W27 ownership. Owner: W22 content-batch test plus W27 regional-NPC data reconciliation. W30 changed only this test's import.
- `tests/game_store.rs:39` (`game_store`, 0 passed / 1 failed): a save without pet state now writes schema 5 while this test expects 4; schema 6 is used when pet state exists. `Housing Core.md` and `Pets Service.md` record the housing/pet save-schema work. Those paths are outside W30 ownership; W30 changed neither `src/game_store.rs` nor `tests/game_store.rs`.

The final run also discovered `tests/trade.rs`, a concurrent builder's target that was left unchanged. Ignored tests were not executed.

| Target | Passed | Failed | Ignored |
|---|---:|---:|---:|
| `unittests src\lib.rs` | 26 | 0 | 0 |
| `unittests src\bin\osfr-gateway.rs` | 0 | 0 | 1 |
| `unittests src\bin\osfr-login.rs` | 0 | 0 | 0 |
| `unittests src\main.rs` | 5 | 0 | 0 |
| `unittests src\bin\soe-probe.rs` | 3 | 0 | 0 |
| `tests\abilities.rs` | 8 | 0 | 0 |
| `tests\account.rs` | 4 | 0 | 0 |
| `tests\character_store.rs` | 8 | 0 | 0 |
| `tests\combat.rs` | 5 | 0 | 1 |
| `tests\content.rs` | 8 | 0 | 0 |
| `tests\content_dungeons.rs` | 3 | 0 | 0 |
| `tests\content_jobs.rs` | 0 | 1 | 0 |
| `tests\content_npcs.rs` | 1 | 0 | 0 |
| `tests\cooking.rs` | 13 | 0 | 1 |
| `tests\cooking_gameplay.rs` | 1 | 0 | 2 |
| `tests\derby.rs` | 11 | 0 | 0 |
| `tests\dev_teleport.rs` | 0 | 0 | 7 |
| `tests\economy_review.rs` | 18 | 0 | 0 |
| `tests\encounter.rs` | 15 | 0 | 2 |
| `tests\encounter_score.rs` | 3 | 0 | 0 |
| `tests\fishing.rs` | 12 | 0 | 0 |
| `tests\fishing_review.rs` | 19 | 0 | 0 |
| `tests\fishing_service.rs` | 12 | 0 | 0 |
| `tests\forging.rs` | 16 | 0 | 1 |
| `tests\forging_gameplay.rs` | 1 | 0 | 3 |
| `tests\game_store.rs` | 0 | 1 | 0 |
| `tests\gameplay.rs` | 0 | 0 | 1 |
| `tests\gateway.rs` | 8 | 0 | 2 |
| `tests\harvesting.rs` | 17 | 0 | 0 |
| `tests\harvesting_border.rs` | 3 | 0 | 0 |
| `tests\harvesting_job_binding.rs` | 3 | 0 | 0 |
| `tests\harvesting_review.rs` | 4 | 0 | 0 |
| `tests\harvesting_reward_catalogue.rs` | 4 | 0 | 0 |
| `tests\harvesting_runtime_border.rs` | 2 | 0 | 0 |
| `tests\harvesting_runtime_tiles.rs` | 3 | 0 | 0 |
| `tests\harvesting_tile_registration.rs` | 9 | 0 | 0 |
| `tests\housing.rs` | 26 | 0 | 0 |
| `tests\housing_native.rs` | 14 | 0 | 1 |
| `tests\housing_service.rs` | 27 | 0 | 0 |
| `tests\inventory.rs` | 16 | 0 | 2 |
| `tests\inventory_v2_fixes.rs` | 3 | 0 | 0 |
| `tests\item_categories.rs` | 3 | 0 | 1 |
| `tests\kart_direct_transport.rs` | 3 | 0 | 0 |
| `tests\kart_entry.rs` | 13 | 0 | 1 |
| `tests\kart_gameplay.rs` | 0 | 0 | 9 |
| `tests\kart_gateway.rs` | 6 | 0 | 0 |
| `tests\kart_lobby_gameplay.rs` | 0 | 0 | 3 |
| `tests\kart_lobby_native.rs` | 10 | 0 | 1 |
| `tests\kart_pickups.rs` | 12 | 0 | 0 |
| `tests\kart_progress_native.rs` | 9 | 0 | 1 |
| `tests\kart_racing.rs` | 13 | 0 | 0 |
| `tests\kart_review.rs` | 12 | 0 | 0 |
| `tests\kart_rewards.rs` | 11 | 0 | 0 |
| `tests\kart_service.rs` | 73 | 0 | 1 |
| `tests\kart_weapons.rs` | 12 | 0 | 0 |
| `tests\login.rs` | 7 | 0 | 2 |
| `tests\minigame.rs` | 7 | 0 | 0 |
| `tests\mining.rs` | 21 | 0 | 0 |
| `tests\mining_dead_board.rs` | 3 | 0 | 0 |
| `tests\mining_idle.rs` | 6 | 0 | 0 |
| `tests\mining_move_detection.rs` | 3 | 0 | 1 |
| `tests\mining_powerups.rs` | 22 | 0 | 0 |
| `tests\mining_review.rs` | 13 | 0 | 0 |
| `tests\mining_shape_parity.rs` | 4 | 0 | 0 |
| `tests\mining_tile_rendering.rs` | 3 | 0 | 0 |
| `tests\mounts.rs` | 9 | 0 | 0 |
| `tests\mystery_chest_codec.rs` | 12 | 0 | 0 |
| `tests\mystery_chest_gameplay.rs` | 0 | 0 | 4 |
| `tests\opening.rs` | 7 | 0 | 2 |
| `tests\opening_abilities.rs` | 4 | 0 | 1 |
| `tests\opening_cave_replay.rs` | 5 | 0 | 0 |
| `tests\opening_gateway.rs` | 0 | 0 | 1 |
| `tests\opening_journal.rs` | 6 | 0 | 2 |
| `tests\opening_world.rs` | 1 | 0 | 0 |
| `tests\pets.rs` | 7 | 0 | 0 |
| `tests\piece_008.rs` | 17 | 0 | 0 |
| `tests\piece_009.rs` | 15 | 0 | 0 |
| `tests\piece_010.rs` | 18 | 0 | 0 |
| `tests\piece_011.rs` | 11 | 0 | 0 |
| `tests\piece_012.rs` | 2 | 0 | 1 |
| `tests\piece_013.rs` | 5 | 0 | 1 |
| `tests\piece_014.rs` | 7 | 0 | 0 |
| `tests\piece_015.rs` | 6 | 0 | 0 |
| `tests\piece_016.rs` | 15 | 0 | 0 |
| `tests\piece_017.rs` | 7 | 0 | 2 |
| `tests\piece_018.rs` | 12 | 0 | 2 |
| `tests\piece_019.rs` | 8 | 0 | 0 |
| `tests\piece_020.rs` | 12 | 0 | 0 |
| `tests\piece_021.rs` | 13 | 0 | 1 |
| `tests\piece_022.rs` | 12 | 0 | 0 |
| `tests\piece_023.rs` | 13 | 0 | 0 |
| `tests\piece_024.rs` | 6 | 0 | 1 |
| `tests\piece_025.rs` | 8 | 0 | 0 |
| `tests\piece_026.rs` | 12 | 0 | 0 |
| `tests\piece_027.rs` | 12 | 0 | 0 |
| `tests\piece_028.rs` | 14 | 0 | 0 |
| `tests\piece_029.rs` | 16 | 0 | 1 |
| `tests\piece_030.rs` | 19 | 0 | 1 |
| `tests\piece_031.rs` | 8 | 0 | 0 |
| `tests\piece_032.rs` | 15 | 0 | 1 |
| `tests\piece_033.rs` | 9 | 0 | 0 |
| `tests\piece_034.rs` | 15 | 0 | 0 |
| `tests\piece_035.rs` | 15 | 0 | 0 |
| `tests\piece_036.rs` | 16 | 0 | 0 |
| `tests\piece_037.rs` | 15 | 0 | 0 |
| `tests\piece_038.rs` | 15 | 0 | 0 |
| `tests\piece_039.rs` | 16 | 0 | 0 |
| `tests\piece_040.rs` | 11 | 0 | 0 |
| `tests\piece_041.rs` | 11 | 0 | 0 |
| `tests\piece_042.rs` | 8 | 0 | 0 |
| `tests\piece_043.rs` | 13 | 0 | 0 |
| `tests\piece_044.rs` | 7 | 0 | 0 |
| `tests\piece_045.rs` | 8 | 0 | 1 |
| `tests\piece_046.rs` | 16 | 0 | 0 |
| `tests\piece_047.rs` | 8 | 0 | 0 |
| `tests\piece_048.rs` | 16 | 0 | 0 |
| `tests\piece_049.rs` | 6 | 0 | 0 |
| `tests\piece_050.rs` | 10 | 0 | 0 |
| `tests\piece_051.rs` | 8 | 0 | 0 |
| `tests\piece_052.rs` | 8 | 0 | 0 |
| `tests\piece_053.rs` | 12 | 0 | 0 |
| `tests\piece_054.rs` | 8 | 0 | 0 |
| `tests\piece_055.rs` | 12 | 0 | 0 |
| `tests\piece_056.rs` | 16 | 0 | 0 |
| `tests\piece_057.rs` | 11 | 0 | 0 |
| `tests\piece_058.rs` | 14 | 0 | 0 |
| `tests\piece_059.rs` | 11 | 0 | 0 |
| `tests\piece_060.rs` | 16 | 0 | 0 |
| `tests\piece_061.rs` | 10 | 0 | 0 |
| `tests\piece_062.rs` | 8 | 0 | 0 |
| `tests\piece_063.rs` | 7 | 0 | 0 |
| `tests\piece_064.rs` | 8 | 0 | 0 |
| `tests\piece_065.rs` | 12 | 0 | 0 |
| `tests\piece_066.rs` | 11 | 0 | 0 |
| `tests\piece_067.rs` | 11 | 0 | 0 |
| `tests\piece_068.rs` | 9 | 0 | 1 |
| `tests\piece_069.rs` | 9 | 0 | 0 |
| `tests\piece_070.rs` | 3 | 0 | 1 |
| `tests\piece_071.rs` | 12 | 0 | 0 |
| `tests\piece_072.rs` | 12 | 0 | 0 |
| `tests\piece_073.rs` | 11 | 0 | 1 |
| `tests\piece_074.rs` | 7 | 0 | 0 |
| `tests\piece_075.rs` | 6 | 0 | 0 |
| `tests\piece_076.rs` | 9 | 0 | 0 |
| `tests\piece_077.rs` | 11 | 0 | 0 |
| `tests\piece_078.rs` | 7 | 0 | 0 |
| `tests\piece_079.rs` | 6 | 0 | 0 |
| `tests\piece_080.rs` | 4 | 0 | 0 |
| `tests\piece_081.rs` | 12 | 0 | 2 |
| `tests\piece_082.rs` | 7 | 0 | 1 |
| `tests\piece_083.rs` | 13 | 0 | 0 |
| `tests\piece_084.rs` | 16 | 0 | 0 |
| `tests\piece_085.rs` | 13 | 0 | 0 |
| `tests\piece_086.rs` | 10 | 0 | 0 |
| `tests\piece_087.rs` | 17 | 0 | 0 |
| `tests\piece_088.rs` | 12 | 0 | 0 |
| `tests\piece_089.rs` | 15 | 0 | 0 |
| `tests\piece_090.rs` | 9 | 0 | 0 |
| `tests\piece_091.rs` | 15 | 0 | 0 |
| `tests\piece_092.rs` | 11 | 0 | 0 |
| `tests\piece_093.rs` | 8 | 0 | 0 |
| `tests\piece_094.rs` | 5 | 0 | 0 |
| `tests\piece_095.rs` | 11 | 0 | 0 |
| `tests\piece_096.rs` | 9 | 0 | 1 |
| `tests\piece_097.rs` | 8 | 0 | 1 |
| `tests\piece_098.rs` | 10 | 0 | 1 |
| `tests\piece_099.rs` | 9 | 0 | 0 |
| `tests\players.rs` | 7 | 0 | 0 |
| `tests\profiles.rs` | 6 | 0 | 1 |
| `tests\quest_engine.rs` | 8 | 0 | 0 |
| `tests\quests.rs` | 7 | 0 | 1 |
| `tests\rewards.rs` | 6 | 0 | 0 |
| `tests\save_schema.rs` | 5 | 0 | 0 |
| `tests\session.rs` | 3 | 0 | 0 |
| `tests\smelting.rs` | 13 | 0 | 1 |
| `tests\smelting_gameplay.rs` | 1 | 0 | 1 |
| `tests\soccer.rs` | 11 | 0 | 1 |
| `tests\soccer_match.rs` | 27 | 0 | 0 |
| `tests\soccer_native_wire.rs` | 8 | 0 | 0 |
| `tests\soccer_review.rs` | 10 | 0 | 0 |
| `tests\soccer_service.rs` | 13 | 0 | 1 |
| `tests\social_chat.rs` | 12 | 0 | 0 |
| `tests\social_lists.rs` | 9 | 0 | 0 |
| `tests\soe_capture.rs` | 0 | 0 | 1 |
| `tests\soe_loopback.rs` | 1 | 0 | 0 |
| `tests\soe_send.rs` | 7 | 0 | 0 |
| `tests\soe_transport.rs` | 10 | 0 | 1 |
| `tests\startup_catalogues.rs` | 6 | 0 | 1 |
| `tests\startup_messages.rs` | 2 | 0 | 1 |
| `tests\startup_selfdata.rs` | 3 | 0 | 1 |
| `tests\startup_snapshot.rs` | 1 | 0 | 1 |
| `tests\startup_world.rs` | 6 | 0 | 1 |
| `tests\trade.rs` | 10 | 0 | 0 |
| `tests\type8_gateway.rs` | 10 | 0 | 0 |
| `tests\type8_harvesting_gateway.rs` | 11 | 0 | 0 |
| `tests\type8_native_repairs.rs` | 8 | 0 | 0 |
| `tests\world.rs` | 7 | 0 | 0 |
| `tests\zone_transition.rs` | 9 | 0 | 1 |
| `Doc-tests osfr_tools` | 0 | 0 | 0 |
Release build succeeded. Binaries stayed in the configured Cargo target directory; none were copied.

```powershell
$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W30'
& ..\..\scripts\cargo.ps1 build --release --bins
```

Result: exit 0; optimized release build finished in 54.14 s. Existing warning: unused `housing::Lot::instance_index`.

Formatting used the private `rustfmt.exe --edition 2024` on `src/lib.rs` and the sixteen converted test files. Ponytail kept the implementation to registration and imports; Caveman kept progress concise. Game-recon, web research, reverse-engineering, and asset-pipeline workflows were not needed for this integration-only task.
