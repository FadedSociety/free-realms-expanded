# Harvesting Lifecycle and Seeded Round

Status: W43 lifecycle adapter, goal render mapping and deterministic local whole-round acceptance are implemented. All requested Rust test targets pass. No native client was launched; the result is not native acceptance or historical balance validation.

## Recovered client facts

| Fact | Evidence |
|---|---|
| Game 111 is Harvesting/Farming, type 8, name 100834, description 94150, icon 908. | Client `Resources/MiniGameData.txt:112`, row `111^8^100834^94150^908^2...`; SHA-256 `8CE3CC61A2AD324912465E8D1785D246CCD680A01712E256FDDF702CE856C252`. Type 8 is Flash in `MiniGameTypeData.txt:9`; SHA-256 `2D200849D113C0F4B071B869774843B2FE9D07095235E7A6E907BE83156E27D0`. Both files are in `lab/working/OSFR.Client/OSFR Client/Resources/`.
| Region 111 produce order is Stickfruit, Stainberry, Tumtumnip, Spiralmint. | Locale description 94150 in the client catalogue and `Harvesting Restoration.md` §Recovered original material. The product values are `411, 413, 416, 403`, from the produce order in `src/harvesting.rs` and client string mappings below.
| Goal labels resolve through `farmingValue<value>`. | Decompiled `GameClient.as:205-217` builds `GameData.sGAME + "Value" + _nValue` before `SetQuota`; `GameClient.as:33-36` receives the movie game type. Client `Resources/CodeStringMappings.txt` maps `farmingValue403 -> 5398`, `farmingValue411 -> 117125`, `farmingValue413 -> 69329`, `farmingValue416 -> 117130` (lines 4786, 4794, 4796, 4799; SHA-256 `1DA9690FCC034657A1CD980CD96FC18CDC143A2FCF2A4373F8DECCD032353E5A`). The pinned locale table resolves those ids to Spiralmint, Stickfruit, Stainberry and Tumtumnip; table SHA-256 `f1f22a30107c824af4b3021c8c7c323c927d3c6a9b82efc387e9eaba6ae62844`.
| Visible bin tiles use the movie's selectable frames. | `core/TileGenerator.as:56-83` registers `SelectableTile + _nIndex` for category 400; `ServerTileGenerator.as:61-71` sends `OnTileInfoMsg(400 + type, type)`; `Bin.as:159-217` creates its displayed tile from the quota value. Therefore region 111 uses frames 11, 13, 16 and 3. Current `Type8Kind` records register those frame indices. The wire `iconId` is stored by `Bin.SetQuota` but does not drive `CreateTile`; the original server's `SD_ICONID` values were not shipped.
| Produce bundle items are 2885, 2887, 2890 and 2877 in that order. | Locale bundle names 23285-23300 and pinned built data `lab/references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json`, SHA-256 `40a3cc79cfd8adfd50dc689e58de18dc14e21bedd73993b6eab0a43d3d95f025`; summarized with provenance in `Harvesting Restoration.md` §Bundle drop identities. No source code from FreeRealms-Legacy was copied.
| Farming movie identity and preload names. | `AssetsW_009.pack` entry 1448 `game_farming.gfx`, CRC32 `bba0be0e`, SHA-256 `02bfc077af1257c140cd9d590257cfc2ab0ba7e45e1814be975f42607342c49f`; `Assets_000.pack` entry 50 is the 97-name Farming preload list. JPEXS 26.3.0 exported the scripts to `lab/decompiled/minigames/harvesting/scripts/`; intake receipt `lab/reports/harvesting-restoration-20261009T020914Z.json` records the reader, hashes and commands.
| Original chain, score and quota behavior. | Farming `GameData.as`, `GameLevel.as`, `GameClient.as` and `GameServer.as` in that export; summarized in `Harvesting Restoration.md`. Chains require 3+ matching adjacent selectable tiles; quota bins count `floor(collected/perReward)` clusters and a win requires every positive quota.
| Shared leave and results packet contract. | `notes/Build Guides/Gameplay/Minigame Framework.md` and `notes/Research/Minigame Lifecycle.md`: ordinary Quit `39/6`; win/loss order movie terminal, `39/18`, `39/47`, `39/19`; automatic acknowledgement `39/7`; actual results Close `26/42`. A run remains terminal until Close. Pre-win leave/disconnect without payout is reconstructed policy (R).

The previous native Harvesting run loaded game 111, but Sky drag and ordinary Leave attempts did not establish accepted input or exit. Developer `realmexit` cancelled the run; its restart preserved state. This is recorded in `notes/Current Progress.md:23`. It proves board launch only, not goal rendering after this source correction, victory, ordinary Leave, payout or Close/retry.

## Reconstructed policy (R)

- The original `MatchThree*Data.txt` server tables were not shipped. No server quota, duration, spawn table, reward-set amount, star formula, instance namespace or XP amount is claimed as original.
- Use a 120-second level timer, matching the two-minute timer shown in the pinned historical Harvesting screenshot (`lab/decompiled/minigames/harvesting/wiki-harvesting-screenshot.jpg`, SHA-256 `118c2f6b998497308f4cfa700e3c36cbb1e75fe7b0eae3da86d0b6fe7b6981d2`). Use 25 tiles per goal and 5 tiles per bundle so the client displays five bundles, as shown by that screenshot's objectives. These are screenshot-backed local choices, not recovered server rows.
- Use fixed seed 2 for deterministic local replay. The region-111 fixture completes in 22 chains and 84 modeled seconds: 21 four-second timer ticks, because the final chain completes before another tick. The seed was selected by local replay, not recovered from the client. Keep the padded 11x11 grid with 10x9 playable cells, wildcard 5%, collectable/blocker/powerup 0%, and one item per completed bundle cluster. Existing border and tile-registration reviews support the shape and kind registrations; original board templates and rates remain unknown.
- Bind the local activity to Chef profile 45. The Chef/Harvesting relationship is supported by the cached Chef page; Farmer is a separate housing-farm progression. The original server job gate is unknown. Award 25 Chef XP and zero coins; the amount and star count 0 are local policy.
- Grant the engine's completed produce transaction once through `rewards::apply_grant`, using `minigame-run:<instance>`. Do not consume materials. On win emit `CollectItem` for each staged produce grant and `FinishMinigame { minigame_id: 111 }`. This local event bridge does not prove that existing Chef objectives advance: current `chef.json` records have empty objective arrays, and save/quest application remains lead integration work.

## W43 implementation

- `prototype/osfr-tools/src/harvesting_lifecycle.rs` adapts the type-8 engine to `MinigameRuns`, including offer/start, movie messages, server tick loss, pre-win Quit, disconnect, terminal duplicates, result Close and fresh retry ids. It returns staged reward and quest changes for the caller to persist atomically before sending replies.
- `prototype/osfr-tools/src/harvesting.rs` exposes `harvesting::lifecycle`; the content mapping keeps goal values 411/413/416/403 and registers matching category-400 frame ids 11/13/16/3. `Type8Goal.icon_id` uses the item catalogue's icon as metadata because original `SD_ICONID` is unavailable; the client draws its visible bin tile from the recovered value/frame path.
- `prototype/osfr-tools/tests/harvesting_lifecycle.rs` drives region 111 with seed 2 and legal chains. It checks all four goal quota messages and frame registrations, completion in 22 chains with positive time remaining and wall time within 120 seconds, produce/XP grant, quest events, duplicate inertness, loss, Quit, disconnect, conservation and Close/retry. The planner permits the final chain to cross a tile quota by fewer than one bundle; it previously stopped below quota and could not finish.

## Verification

Formatting: private rustfmt.exe --edition 2024 and --check passed for `src/harvesting.rs`, `src/harvesting_lifecycle.rs`, `tests/harvesting.rs`, `tests/harvesting_border.rs`, `tests/harvesting_lifecycle.rs`, `tests/harvesting_review.rs`, `tests/harvesting_tile_registration.rs` and `tests/type8_harvesting_gateway.rs`.

The first requested test build was blocked by in-progress edits in unowned shared source (src/gateway.rs, then src/derby.rs). After those shared errors cleared, the focused targets compiled. The first full run exposed stale quota and seed assertions in owned tests; those expectations now match the candidate settings. Final command from prototype/osfr-tools, with CARGO_TARGET_DIR=lab/build/W43, exited 0:

```powershell
$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W43'
& '..\..\scripts\cargo.ps1' test --quiet --test harvesting --test harvesting_border --test harvesting_job_binding --test harvesting_lifecycle --test harvesting_review --test harvesting_reward_catalogue --test harvesting_runtime_border --test harvesting_runtime_tiles --test harvesting_tile_registration --test minigame --test type8_harvesting_gateway
```

Result: all 67 tests across 11 binaries passed (seeded lifecycle 4/4, generic minigame 7/7, and type-8 Harvesting gateway 11/11). Existing warnings include unused path-included fixture helpers and the unrelated `housing.rs::instance_index` dead-code warning. No test failed. No native client was launched.

## Native test plan for the lead

1. After registering the adapter in the lead-owned gateway/save path, use an isolated disposable game-111 content overlay and Chef45 character. Preserve existing save and client hashes.
2. Check the four named bins and matching tile art. Play normal legal chains to all four five-bundle targets; capture accepted movie payloads and the actual goal counters. Re-test Sky drag only as a separate input path; prior attempts were inconclusive.
3. On win, verify one ordered result sequence, automatic `39/7`, one bundle grant per completed cluster, one Chef XP grant, the objective events, persisted state after reload, `26/42` Close and a retry with a fresh instance id.
4. Separately verify timer loss and ordinary `39/6` Leave. Confirm no materials are consumed and no item/XP grant occurs; check the client returns through its ordinary UI. Preserve disconnect and duplicate-result probes.

Native board geometry, accepted chain input, ordinary Leave, timer loss, complete victory, payout persistence, result Close/retry, the server's `SD_ICONID` values and quest advancement remain open. Client label strings and visible tile-frame mapping are recovered above, but native rendering after this source correction is unverified. The separate housing-farm completion receipt is not evidence for this Harvesting minigame.
