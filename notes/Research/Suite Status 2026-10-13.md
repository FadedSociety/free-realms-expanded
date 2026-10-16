# Ordinary Test Suite Status — 2026-10-13

## Result

Successful run: exit 0. Across 185 Cargo targets: 1781 passed, 0 failed, 87 ignored. Failed tests: none. The 87 ignored tests were left ignored; no --ignored gates were run.

## Changes

- `tests/game_store.rs`: updated the save migration assertion from schema 3 to schema 4. `game_store::SCHEMA_VERSION` is private, so this integration test retains the current numeric assertion without widening the source API.
- `tests/economy_review.rs`, `tests/housing_service.rs`, `tests/piece_045.rs`, `tests/piece_081.rs`, and `tests/type8_harvesting_gateway.rs`: supplied `stored_count: 0` and widened test values with `i64::from(i32::MAX)`.
- `tests/housing_service.rs`, `tests/piece_045.rs`, and `tests/piece_081.rs`: self-body fixture encoders retain the i32 time field and use checked `i64` to `i32` conversion. This preserves their existing fixture bytes and behavior.
- No expected wire bytes changed. The existing 38/3 byte-layout checks remain in place; these edits only repair save literals and the self-body test encoders.
- `tests/startup_selfdata.rs` and `src/startup_selfdata.rs` were not edited. No `src/` file needed a literal fix.

## Evidence and provenance

- The persisted `stored_count`, `i64` `last_cast_time`, schema-4 migration, and reconstructed missing-field default are recorded in `notes/Research/Inventory Search Crash.md`, Integration, lines 105–106. Schema 4 is local save-format policy, not a recovered client schema.
- Recovered 38/3 width and order: that note records byte-checked client functions `0x008D8350` and `0x008B25E0`, including five 32-bit fields followed by eight bytes, lines 110–112. Semantic assignment of the last two fields is marked reconstructed (R).
- The self-body timestamp remains i32: the note records `0x008FA4B0` calling `0x008CB5A0`, whose byte-checked read consumes one i32 and combines it with the client tick, line 111. The test encoders therefore convert with a checked bound.
- No original behavior or numeric game rule was inferred from these mechanical test updates.

## Commands and results

Working directory: `project/prototype/osfr-tools`.

```powershell
$env:CARGO_TARGET_DIR = '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W14'
& '..\..\scripts\cargo.ps1' test --no-fail-fast
```

An initial in-process `Tee-Object` capture aborted when Windows PowerShell promoted Cargo's stderr progress line (`Compiling version_check v0.9.5`) to `NativeCommandError`; it produced no log and is not counted as a test result. The complete runs below used child PowerShell with stdout and stderr redirected to the report directory.

First captured run exited 101 during compilation. It reported `error[E0308]` at `tests/housing_service.rs:287`: expected `i32`, found `i64`. The checked conversion above fixed it. The rerun exited 0.

The full logs are `lab/reports/suite-20261013T205140Z/ordinary-suite.stdout.log`, `ordinary-suite.stderr.log`, `ordinary-suite.exit.txt`, `ordinary-suite-rerun.stdout.log`, `ordinary-suite-rerun.stderr.log`, and `ordinary-suite-rerun.exit.txt`. Per-target counts are also in `lab/reports/suite-20261013T205140Z/target-counts.tsv`.

## Prior-failure comparison

- The prior W09 inventory note already records the `game_store` assertion failing with actual schema 4 versus expected 3 (`notes/Research/Inventory Search Crash.md`, lines 120–121). The W14 snapshot also contains the old expectation at `tests/game_store.rs:38`.
- The snapshot has `InventoryItem.last_cast_time: i32`; its `housing_service` fixture writes that value into an i32 array. The current model had widened the value to i64 before W14, exposing the compile mismatch in the current checkout. W14 repaired the fixture with a checked conversion.
- The successful rerun had no failing tests. No additional failure needed snapshot classification.

## Per-target pass/fail counts

Ignored counts are included because ordinary Cargo runs report them; they were not executed.

| Target | Passed | Failed | Ignored |
|---|---:|---:|---:|
| `unittests src\lib.rs` | 26 | 0 | 0 |
| `unittests src\bin\osfr-gateway.rs` | 0 | 0 | 1 |
| `unittests src\bin\osfr-login.rs` | 0 | 0 | 0 |
| `unittests src\main.rs` | 5 | 0 | 0 |
| `unittests src\bin\soe-probe.rs` | 3 | 0 | 0 |
| `tests\account.rs` | 5 | 0 | 0 |
| `tests\character_store.rs` | 8 | 0 | 0 |
| `tests\combat.rs` | 5 | 0 | 1 |
| `tests\content.rs` | 5 | 0 | 0 |
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
| `tests\game_store.rs` | 1 | 0 | 0 |
| `tests\gameplay.rs` | 0 | 0 | 1 |
| `tests\gateway.rs` | 7 | 0 | 1 |
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
| `tests\housing_service.rs` | 19 | 0 | 0 |
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
| `tests\mining.rs` | 21 | 0 | 0 |
| `tests\mining_dead_board.rs` | 3 | 0 | 0 |
| `tests\mining_idle.rs` | 6 | 0 | 0 |
| `tests\mining_move_detection.rs` | 3 | 0 | 1 |
| `tests\mining_powerups.rs` | 22 | 0 | 0 |
| `tests\mining_review.rs` | 13 | 0 | 0 |
| `tests\mining_shape_parity.rs` | 4 | 0 | 0 |
| `tests\mining_tile_rendering.rs` | 3 | 0 | 0 |
| `tests\mystery_chest_codec.rs` | 12 | 0 | 0 |
| `tests\mystery_chest_gameplay.rs` | 0 | 0 | 4 |
| `tests\opening.rs` | 7 | 0 | 2 |
| `tests\opening_abilities.rs` | 4 | 0 | 1 |
| `tests\opening_cave_replay.rs` | 5 | 0 | 0 |
| `tests\opening_gateway.rs` | 0 | 0 | 1 |
| `tests\opening_journal.rs` | 6 | 0 | 2 |
| `tests\opening_world.rs` | 1 | 0 | 0 |
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
| `tests\profiles.rs` | 6 | 0 | 1 |
| `tests\quests.rs` | 7 | 0 | 1 |
| `tests\rewards.rs` | 6 | 0 | 0 |
| `tests\save_schema.rs` | 3 | 0 | 0 |
| `tests\session.rs` | 3 | 0 | 0 |
| `tests\smelting.rs` | 13 | 0 | 1 |
| `tests\smelting_gameplay.rs` | 1 | 0 | 1 |
| `tests\soccer.rs` | 11 | 0 | 1 |
| `tests\soccer_match.rs` | 27 | 0 | 0 |
| `tests\soccer_native_wire.rs` | 8 | 0 | 0 |
| `tests\soccer_review.rs` | 10 | 0 | 0 |
| `tests\soccer_service.rs` | 13 | 0 | 1 |
| `tests\soe_capture.rs` | 0 | 0 | 1 |
| `tests\soe_loopback.rs` | 1 | 0 | 0 |
| `tests\soe_send.rs` | 7 | 0 | 0 |
| `tests\soe_transport.rs` | 10 | 0 | 1 |
| `tests\startup_catalogues.rs` | 6 | 0 | 1 |
| `tests\startup_messages.rs` | 2 | 0 | 1 |
| `tests\startup_selfdata.rs` | 3 | 0 | 1 |
| `tests\startup_world.rs` | 6 | 0 | 1 |
| `tests\type8_gateway.rs` | 10 | 0 | 0 |
| `tests\type8_harvesting_gateway.rs` | 11 | 0 | 0 |
| `tests\type8_native_repairs.rs` | 8 | 0 | 0 |
| `tests\zone_transition.rs` | 9 | 0 | 1 |
| `doc-tests osfr_tools` | 0 | 0 | 0 |
