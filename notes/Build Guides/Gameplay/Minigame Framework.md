---
type: build-guide
section: Gameplay
updated: 2026-10-12
---

# Minigame Framework

Part of [[Build Guides]] › Gameplay. Related: [[Kart Racing]], [[Trading Card Game]], [[Chess]], [[Soccer]], [[Fishing]], [[Matchmaking and Lobby]].

<!-- NARRATIVE START -->
## How it works

**How to read this.** Confidence words (`confirmed`, `likely`, `speculative`) are the native-map labels for a function. Source tags: *client* = read from the shipped executable in a shard or in this pass (pseudocode is a field-level reading of the Ghidra export, an inference, not a captured fact); *client Lua* = the client's own UI script (`ScriptsBase.lua`, decompiled in the lab, not redistributed); *client AS2* = ActionScript exported from the shipped Flash movies (`score-screen`, the cooking, forging, smelting, mining and harvesting movies); *client resource* = a data file from the shipped `Resources` folder (re-counted in this pass); *client locale* = the shipped text table; *native run* = our Rust server driven against the real client (this proves what the client accepts, never what the original server did); *video* = a reviewed frame record (no player names or chat are recorded); *wiki* = the fan wiki. Precedence is client, then video, then wiki. **(R)** marks a reconstruction: our own server policy or an inference the client alone cannot settle. No original server code exists. OSFR-held packet captures are not used. Related: [[Kart Racing]], [[Trading Card Game]], [[Chess]], [[Soccer]], [[Fishing]], [[Demo Derby]], [[Matchmaking and Lobby]], [[Items and Inventory]], [[Combat and Abilities]], [[Quests and Objectives]], [[Rewards and Collections]].

### Overview

The minigame framework is the part of the client that every "game" shares, whatever it is underneath. It is one manager object (`ClientMiniGameManager`), one wire family (opcode 39), four data tables, one Lua start screen and one Lua/Flash score screen. It does not play any game. It decides which game state exists, shows the start screen and goals, relays messages to a Flash movie or native engine, and, when the server says the game is over, shows a results screen, a prize wheel and the reward lists, then returns the player to the world.

- **Three kinds of game plug into it.** (1) *Flash movies* driven entirely by messages the server sends and the movie sends back through the framework: the match-three family (TYPE 8: Mining, Harvesting, Forging, Mail Sorting, Artifact Hunt), the "mimic" family (TYPE 9: Cooking, Smelting, Feeling the Beat, Hack 'n' Slash), tower defence (TYPE 14), mahjong (TYPE 15), spot-the-difference (TYPE 1009), the daily wheel (TYPE 22), Chess (TYPE 6, a movie with the native `ChessLib.dll` engine behind it) and Checkers (TYPE 7, a movie). (2) *Native engines* with their own packet family and processor object: Kart Racing (33), Demolition Derby, Soccer (76), Fishing, Pirates ship combat (94), Battle Mages, and the card games (the card client is a separate DLL). They still use the framework for launch, goals, HUD and results. (3) *Server-run encounters* that need nothing but the generic packets: the 152 TYPE 1 quest games (foot races, sheep, shopping lists), the 91 TYPE 2 delivery games, and every combat dungeon and battle. These have no client-side game code at all. (client resource, client, client Lua)
- **What the player sees.** A start panel (game name, a difficulty row of up to five coin icons, "Prizes: win the game and you could win one of these prizes!", a Go button) -> a goals panel and HUD (star counter, score, timer) -> the game -> a results screen with page tabs (rewards, statistics, score) that ends in a Close button. Many wins (dungeons, Artifact Hunt, the farm shopping-list games) end in a ring-shaped **prize wheel**. The manager also holds a "play more" prompt (activity category, image, message and header strings, `009b7810`) and fires first-completed and first-unlocked events (39/65, 39/66) for the new-player flow. (video, client Lua, client)
- **Rewards.** The results screen only *displays* reward data the server already sent; coins, stars, items and quest progress are granted by ordinary server updates (inventory, wallet, job experience). Coins and stars appear as two synthetic reward rows, items as entry rows, and the wheel's landing slot is chosen by the server before the player spins. (client Lua, client)
- **Registry size.** The shipped tables hold 783 games, 22 declared type ids (20 with games, 2 declared and empty), 20 groups and 610 group links. Only the type ids give a game its behaviour; names, difficulty and prize text come from the locale. (client resource)
- **What our server has proven.** The real client has completed Cooking (row 46), Smelting (row 77) and Forging (row 139) end to end against the Rust server: info, start, movie handshake, rounds, win, the 39/18, 39/47, 39/19 results burst, the visible results page, the player's Close, and an atomic recipe reward (ingredients consumed, output granted, quest advanced, all in one commit). The cave combat results page and a loss page were also shown natively. A Mining board has run natively through one accepted drag and a clean quit; Harvesting is code-tested only. Everything about reward *amounts*, level tables and the order of the reward packets is reconstructed. (native run, vault: Native Cooking Finish and Mae 171251, Native Gear Up and Smelting 173814, Native Forging Victory 181715, Native Mining Score 121834)

### How the client implements it

#### Layers

| Layer | Role | Evidence |
|---|---|---|
| `ClientMiniGameManager` (ctor `009bc410`, vftable `0180f130`; derives `GuiEventHandler` and `DelayedAssetLoader::StatusHandler`) | One per client. Owns the state list, the four definition tables, the current group record, the objective runtime (`00c43010`), the pending "play more" slots and a small load-state machine. | confirmed |
| `MiniGameState` records (hash list `HashList<MiniGameState,10,-1>` at manager `+0x10`; list head = manager `+0x18` = "current state") | One per live game instance, keyed by the **instance id** (the first header id, key at record `+0x918`, ten buckets). Holds game type, goals, score rows, three reward bundles, timing and flags. Insert `009b16e0`, lookup `009ad0b0`, remove `009baba0`. | confirmed / likely |
| Dispatchers `009bfea0` (main) and `009bf890` (fallback) | Switch on the family-39 sub-opcode. The main switch has explicit arms for 1, 2, 15, 16, 18, 19, 20, 21, 23, 44, 45, 47, 50, 51, 62, 67 and 68. **Its default arm, which carries 14, 17, 22, 40, 41, 54, 65 and 66, runs only when a state with the header's instance id exists**, and several explicit arms (21, 23) need a current state, so most messages are ignored until a 39/16 has created the state. | confirmed |
| Lua `MiniGameHandler`, `MiniGameStatusHandler`, `MinigameStartScreen`, `HandlerMiniGameStart` | Start screen, ready/not-ready, quit, round-end screens, the per-type constants. | client Lua |
| Lua `MiniGameFlash` + native `MiniGameFlashC` (`00c08280`) | The bridge to a Flash movie window: `Show/ShowMicro`, `OnMessage`, `OnKeyPressed`, pause/resume, and about 60 `commands` the movie may call. | client Lua, confirmed |
| Lua `ScoreScreen` + `scoreScreen.gfx` | The single results window used by every game type (seven page states). | client Lua, client AS2 |
| Data sources `BaseClient.MiniGame`, `.LastMiniGame`, `.MiniGameData`, `.MiniGameGroup`, `.MiniGamePortal`, `.MiniGameGoals`, `.MiniGameScore`, `.MiniGame.Trophies`, `.MiniGame.Reward(.Entries)`, `.MiniGame.MemberReward`, `.MiniGame.RewardPreview(.Entries)`, `MiniGame.Bosses` | What the Lua reads. Their column lists are in Data and formats. | confirmed (S0392, S0393) |

#### Type registry

The client never switches on a table row to decide behaviour; it switches on the **game type id** carried in the 39/16 info record (record `+0x250`, copied into the state's first dword by `00c42c60`) and cross-checks the type table for flags. Type ids are named by the client Lua enum in `MiniGameStatusHandler` (NONE -1, TUTORIAL 0, SAMPLE 1, SCAVENGER_HUNT 2, FLASH 3, COMBAT 4, HIDDEN_OBJECT 5, CHESS 6, CHECKERS 7, MATCH_THREE 8, MIMIC 9, RACING 10, DEMO_DERBY 11, SOCCER 12, PIRATES 13, TOWER_DEFENSE 14, TILE_MATCHING 15, TRADING_CARD_GAME 16, PLATFORMER 17, FISHING 21, WHEEL 22, SPOT_THE_DIFF 1009). Ids 18, 19, 20, 23 and 1011 have no Lua name but occur in the data. The full table is under Data and formats.

Where the code branches on type (all *client*, pseudocode):

- `DispatchMiniGamePacket` `009bf890`: sub-opcode `0x28` (start card game) goes to the object at client `+0x108` for type 23 (Treasure War) and `+0x104` for type 16 (card game), both filled in `PlayClient::Initialize` `0092b170`. This is the routing the Trading Card Game guide describes.
- `TickMiniGameManager` `009bf5e0` (manager state 3): for types 16 and 23 it first enters manager state 4 and calls the card manager's start slot (`+0x60` of the object at client `+0x104` or `+0x108`) unless `008c7c20(9)` is already true (likely); every other type goes idle and reaches `RequestStartMiniGame` automatically unless the info record asks for a Go click.
- `HandleMiniGameStart` `009beb70`: types 1, 2, 17 (`0x11`) and 22 (`0x16`) set manager state 3 immediately (no asset preload step); other types with a non-empty preload string set state 1 and wait for the delayed asset loader.
- `ShowMiniGameHud` `009b6aa0`: types 10, 11 and 12 hide the main HUD and the dock and show the star counter (10 and 11 also position the race star counter, 11 also shows the flash score); type 19 hides the main HUD and dock only; every other type calls `HUD:showMinigame` with three flag bytes from the state.
- `IsMiniGameFlashType` `008a5850` (used by the action bar to decide whether a key press goes to the movie as `MiniGameFlash:OnKeyPressed`): true for types 3, 6, 7, 8, 9, 14, 15, 16, 22, 23 and 1009. This **differs** from the type table's `IS_FLASH` column (3, 8, 9, 14, 16, 23). Chess, Checkers, mahjong, the wheel and spot-the-difference are movies the client treats as Flash although the table does not flag them.
- `SetMiniGameEndData` (Lua, `MiniGameHandler`): maps the type to a results page or to no page (table below). `ScoreScreen.m_ValidRewardGameTypes` = 4, 6, 7, 8, 9, 14, 16 and 1009: only these show reward, statistics and score tabs.
- State type `0x3eb` (1003) gets an extra reset call when its state is removed (`009bfea0` case `0x13`). No data row has type 1003. (open question)

#### Manager and state lifecycle

A game instance moves through these steps on the client. Field offsets are from pseudocode.

| Step | Trigger | What the client does |
|---|---|---|
| Created | S>C **39/16** (info) -> `HandleMiniGameJoinGame` `009bfdb0` -> `HandleMiniGameStart` `009beb70` | Logs `MiniGame:JoinGame`, hides the marketplace, sends a dismount request if connected, drops any previous state, creates the state keyed by the header's instance id (`009b9680`), fills it from the info record (`00c42c60`), copies the three reward bundles, builds the goal list from the record's objective array, and emits `HandlerMiniGameStart:setNotReady`. Client `+0x700` becomes 2 for an ordinary game or 1 when the state is an encounter (`+0x61` set). |
| Waiting / loading | Manager state at `+0x2bea0` | 1 = wait for the delayed asset loader (the info record's preload string, queued through `00ae79b0`; `PRELOAD_FILE` is empty in every table row, so the server supplies it; likely); 3 = ready; 4 = a card manager is loading (types 16, 23); 0 = idle; 5 and 6 = teardown (`009b6490`, removal of the state) and not otherwise traced. |
| Start screen | `:show` (after the manager's countdown at `+0x54`) | Shows the Lua start screen unless info byte `+0x26d` is set. A state whose info byte `+0x267` (state `+0x907`) is clear is "micro": it auto-requests start (`009be7d0`) and the movie opens with `ShowMicro`; otherwise the player's Go sends the request. (pseudocode of `009beb70`, `009bf5e0`, `009bd650`) |
| Start request | Go, or auto | C>S **39/5** `[instance, -1, -1]` (`RequestStartMiniGame` `009be7d0`; the writer `009bdc60` substitutes the current instance when the first id is -1). Fires `GameEvents:OnPlayerStartMiniGame`. |
| Started | S>C **39/17** (ordinary states) or **41/2** (states flagged as encounters) | `HandleMiniGameGameStarted` `009b9550` (logs `MiniGame:OnGameStarted`) looks the state up by the header id, and if it has not started sets `+0x62`, stamps the start time (`+0x68/+0x6c`), calls `ShowMiniGameHud` and, for types 16 and 23, the card manager's slot `+0x54`. For encounter-flagged states the same job is done by `MiniGameState_StartActive` `009b9500`, which the encounter dispatcher calls on 41/2 (`00aa36c0` case 2) when `+0x61` is set, so those games start without a 39/17. For Flash games the server also sends **26/12**, whose handler (`00aa0a10` -> `StartFlashMiniGameUi` `009bd650`) calls `<Handler>:Show` or `<Handler>:ShowMicro` with the movie file. |
| Playing | game messages | See below. |
| Game over | S>C **39/18** | `ApplyMiniGameGameOverResult` `009ad480` stores the won byte at state `+0x63` and three dwords at `+0x70/+0x74/+0x78` (stars, elapsed seconds, total score in the native runs). It does **not** open the results screen. |
| Scores | S>C **39/47** | `009bb030` reads the row count and rows (`009b82d0`, `009b2b30`) into the state's score block (`+0x2cc`). |
| Removed | S>C **39/19** | Dispatcher case `0x13` -> `LeaveMiniGame` `009bf190` -> `ShowMiniGameEndDialog` `009b7170`. That function runs only if state byte `+0x908` (the info record's fifth flag byte) is set and the dialog has not been shown (`+0x2c0`). It settles unfinished goals (win: goals become Completed; loss: Failed), counts completed over total for the primary goals and for the bonus-flagged goals (record byte `+0x28`), converts `+0x74` to minutes and seconds and passes status, the two ratios, time, stars (`+0x70`) and total (`+0x78`) to Lua `HandlerMiniGameEnd:setMiniGameEndData`, then `ScoreScreen:Show(status, type)`. The client automatically answers with **39/7** (an acknowledgement, not the player's Close). |
| Closed | Player clicks Close | `ScoreScreen:ExitScreen` -> `Ui.LeaveMiniGameOrEncounter` (`00c172c0` -> `008d2730` -> `009bd240`): sends **41/109** only when the state's `+0x61` flag is set, removes the local state, asks for an activity-list refresh and sends **26/42** (4 bytes `1a 00 2a 00`, `PlayClient::SendClosedMinigameEndScreen` `0092f300`). |

State flags used above: `+0x61` encounter / in progress (set from the third argument of `009beb70`; ordinary joins pass 0), `+0x62` started, `+0x2b8` set by `009ad420`, `+0x2c0` end dialog showing and `+0x2c1` leaving (`009ad830` clears the first and sets the second when the Lua reports `EndGameDialogClosed`).

#### During play

- **Flash movies** talk to the server only through the framework. The movie's own `SoeGameFramework` (all five exported movies carry the same file, `SoeGameFramework.as`) calls `fscommand("FR_event", ...)`. `SendMsg( "<Name>\t<arg>...")` goes to Lua `MiniGameFlash.commands.SendMsg`, then `MiniGameFlashC:SendMessage` (`00c08480`) and `SendMiniGameFlashPayload` (`009be6b0`), which builds a **39/14** payload carrying the string **including its NUL** (disassembly `add eax,1` at `00c08519`) and sends it only if the manager count is positive. The reverse path is S>C 39/14 -> `009b89e0` -> Lua `MiniGameFlash:OnMessage(payload)` -> the movie's `_root.OnMessage`, which splits on tab and passes the first token as the function name with up to seven arguments (`undefined`, `true`, `false` and numbers are converted). (client, client AS2, native run)
- **Movie-to-server names** (from `SoeInterfaceFreeRealms.as`): `FRServer_Connect`, `FRServer_ReadyForGameData`, `FRServer_StatUpdate` (stage type, stat type, count), `FRServer_Pause`, `FRServer_RoundTransitionPause`, `FRServer_CompleteLevel`, `FRServer_StopLevel` (level, winner flag), `FRServer_ScoreIncrement`, `FRServer_ScoreSet`, `FRServer_GameWon` (score, quantity), `FRServer_GameLost`, `FRServer_GameQuit`, `FRServer_GameOver`, `FRServer_GameClose`. The movie writes `FRServer_StatUpdate` with a missing closing quote, so the last numeric field arrives with a literal `)` (seen on the wire in native Cooking and Smelting runs). The movie's `OnGameWonMsg` calls `EndOfGame`, which only emits `FRServer_GameOver`; it does not open the results screen. (client AS2, native run)
- **Server-to-movie names** (from the shared framework and the movies' network classes): `OnConnectMsg`, `OnServerReadyMsg`, data replies (`OnGameDataMsg`, `OnLevelDataFromFR`, `OnReceivedAllDataFromFR`), `OnStartGameMsg(level)`, `OnStartLevelMsg`, `OnLevelInitMsg`, `OnNextLevelMsg`, `OnStopLevelMsg`, `OnPauseMsg(state)`, `OnScoreMsg`, `OnGameWonMsg`, `OnGameLostMsg`, `OnGameStoppedMsg`, `OnShowEndRoundScreenMsg`, `OnSkipStageOrLevelMsg`, `OnLevelEventMsg`, `OnGamePlayTimeMsg`, `OnBoostStatusMsg`, `OnLevelHistoryMsg`, `OnActiveMsg`; the match-three movies add chain, build-map, quota, tile and random-queue messages. The framework itself: `OnNextLevelMsg` compares the level goal with the score and calls `GameLost` or `FinishLevel`; `GameWon` and `GameLost` are ignored unless a game is in progress; a dropped connection reaches none of them (`GameClientDisconnect` is empty in every movie). (client AS2)
- **Other commands a movie may call** (Lua `MiniGameFlash.commands`): sound (`PlayMusic`, `PlaySound`, `LoadSound`, ...), HUD (`StarCounterShow/Increment`, `ScoreShow/Increment/Set`, `LevelShow/Increment/Set`, `TimerSet/Start/Stop/Increment`, `StopwatchSet/Start/Stop`, `SetHealth`, `Task*`), windows (`SetWindowRequired`, `ShowTutorial`, `ShowTutorialPage`, `ShowErrorMessage`, `Resize`, `Offset`), round screens (`ShowLevelEndScreen`, `AddLevelEndScreenData`, `HideLevelEndScreen`, `ShowEndStageScreen`), data (`LoadData`, `Translate`, `StringByName/ById/Batch`, `requestDataSourceUpdate`, `RequestTint`), and exits (`Quit`, `endGame`, both calling `Ui.QuitMiniGame`). `LoadData(filename)` reads a **client file** (`ReadTextFileToIString`, `00c082d0`); it is not a server request. `ReadyToStartGame` marks the movie loaded and, if no how-to window is open, tells the movie the tutorial closed. (client Lua, client)
- **Native engines** keep their own state machines and families (see the per-game guides). The framework contributes the launch handshake, the shared HUD and the results. `WaitForWorldReady` (`00936d00`) holds the zoning barrier until the race, derby, soccer, Battle Mages, fishing and ship-combat processors report ready, then sends the client's finished-loading packet. (client, S0247)
- **Goals and objectives.** Each state owns an array of 0x110-byte goal records built from the info record's goal array (`00c42870`, `00c42bd0`). Family 45 sub-opcodes 1 to 4 (and 5, unhide) update them while a state exists: status 1 NotStarted, 2 InProgress, 3 Completed, 4 Failed, 5 failure-condition, 6 MembershipRequired (`00c42440`). Completing a goal adds the amount to six per-category counters on the state and raises `ObjectiveComplete` (`009b6700`). These are a different system from quest objectives; see [[Quests and Objectives]]. The score data source's total row is *not* computed from these counters; the server sends it in 39/47. (client)
- **Other live messages.** 39/21 raises `StarCounter:IncrementCounter` by the increase over the state's stored count (ignored when the flag byte is set); 39/20 carries a float for `GameEvents:OnSetGameTimeScalar`; 39/23 carries current and maximum knockouts for `GroupHandler:SetKO` (reader `009b87a0`); 39/15 carries a locale string id and a flag and prints the resolved text; 39/62 updates the `MiniGame.Bosses` table (see [[Combat and Abilities]]); 39/44 shows a marketplace advertisement at the start screen (`ShowMiniGameMarketplaceAd` `009b6fe0`, see [[Coin Store and Purchases]]). (client)

#### Results, score screen and rewards

All game types share one window, `Main.wndScoreScreen` (`scoreScreen.gfx`). It has seven page states: statistics (0), rewards (1), score (2), race results (3), derby results (4), loot wheel (5) and soccer (6).

1. **Opening.** `MiniGameHandler:SetMiniGameEndData(status, ...)` reads the current game's type from `BaseClient.MiniGame` and opens `ScoreScreen:Show(status, page type)` for these types: 1 and 4 -> 4; 6, 7, 8, 9, 13, 14, 16 and 1009 -> the same number. Types 10 and 11 open nothing here (their own `RaceResults:Show` and `DerbyResults:Show` call `ScoreScreen:Show(3, 10)` and `Show(4, 11)`), soccer calls `SoccerResults:Show(winLose)` -> page type 12, type 21 (fishing) quits straight to the world, type 2 quits and leaves, and anything else quits and shows the main HUD. (client Lua)
2. **Status.** The first argument is the status: 1 = win (headline "YOU WIN"), 3 = race place (headline is the ordinal place), 4 = derby complete, other = lose (the `YouLose` headline; the native loss run showed the TRY AGAIN title). Win, race and derby statuses play the win music, other statuses the lose music. (client Lua)
3. **Pages are built from data, not from the packet.** `SetStates` adds the rewards tab only if the reward list is non-empty, the statistics tab only if there are stat rows, and the score tab only if there are score rows, and only for the valid reward types above. **If it adds none, the movie closes itself** (`initializeState` calls `close`, which sends `Hide`, which exits to the world). A win with empty reward, stat and score data therefore flashes away without the player seeing it; this was seen natively for the cave combat result and fixed by sending a real total-score row. (client Lua, client AS2, native run)
4. **Rows.** Score rows come from `BaseClient.MiniGameScore`. A row with Score Type above 100 goes to the statistics tab; the others go to the score tab. Type 4 is the total (`GetTotalScore` and the headline number both read the Points column of that row); type 3 is the goal separator. The row's name is a code string key that the client maps to a locale id (for example `scoreTotalScore` -> 117296, `scoreEnemiesDefeated` -> 117298). (client Lua, client AS2, client resource)
5. **Rewards tab.** `ParseRewards` reads `BaseClient.MiniGame.Reward`: coins (column 3) and stars (column 2) each become a synthetic row (icon 4809 / icon 9) only when the quantity is positive **and the status is a win**; item rows come from `Reward.Entries` where Type is 1 and Hidden is not 1. An entry flagged Show Separated is shown in its own slot. A "Members Only Bonus" block comes from `BaseClient.MiniGame.MemberReward` (coins). (client Lua; video shows the coin and star lines and the members block)
6. **Prize wheel.** See next section.
7. **Grant.** No packet in this path grants anything. Coins, stars and items must arrive as wallet, experience and inventory updates; a "collected items" sentence is built locally from the reward entries (`ShowCollectedMiniGameItemRewards` `009b3970`). (client)
8. **Race, derby and soccer tabs** are fed by their own data sources (`BaseClient.VehicleRaceProcessor.RaceResults`, `BaseClient.VehicleDemolitionDerbyProcessor.DerbyResults`, soccer team and player stats), not by 39/47. The race headline is the ordinal place ("1ST PLACE!"). (client Lua, video)
9. **Close.** The movie's `exitScreen` or `Hide` events route to `ScoreScreen:ExitScreen`, which disconnects the data sources, hides the window, calls `Ui.LeaveMiniGameOrEncounter` and restores the HUD (housing instances restore the housing UI instead). Soccer's half-time board (`-12`) closes without leaving. (client Lua)

Natively proven results presentation: after a crafting win the order on the wire is `OnGameWonMsg` (to the movie), then **39/18, 39/47 (one `scoreTotalScore` row), 39/19**, then the client's automatic 39/7, then, when the player clicks the original Close, a 4-byte 26/42. The page reads "YOU WIN!", the recipe name and "TOTAL SCORE". The server must keep terminal state until the Close arrives. (native run)

#### Prize wheel

- **Data.** The wheel is populated when the score screen finishes its transition (`PopulateLootWheel`). Candidate prizes are the rows of `BaseClient.MiniGame.RewardPreview.Entries`; the preview bundle (`BaseClient.MiniGame.RewardPreview`) adds a Coins slot (icon 22663, label locale 6513) and an Experience slot (icon 9, label locale 426522) when those amounts are positive. The wheel is shown only when the status is above 0 and there is at least one *entry* row; otherwise the screen skips to the normal pages. Header strings are "Congratulations!", "You won" and "Click here!". (client Lua, client locale, video: eight-slot wheel, centre item, prize pop-ups)
- **Landing slot.** The spin is theatre. The server sends **39/45** (`MiniGameLootWheelSetItemToLandOnPacket`) *before* the spin; `StopMiniGameLootWheelAt` `009b6da0` compares the awarded entry with the state's preview entries and raises `ScoreScreen:StopLootWheelAt(index)`. By pseudocode the index is the entry's position; when no entry matches, the Coins slot is index "entry count", and the Experience slot comes after it. The Lua stores the index; the player's click (`OnLootWheelSpinRequest`) sends **nothing**, it only starts the animation toward the stored index. (client Lua, client AS2, client)
- **Stop report.** At spin time the Lua adds the landing item (or the Coins or Stars row) to the rewards list and starts the animation. When the wheel stops the movie sends `OnLootWheelRotationStopped`; Lua calls `Ui.SendLootWheelRotationStopPacket()` and moves on to the normal pages. The packet is **C>S 39/46** (constructor `009aa0c0`, sender wrapper `009bda80`, no recorded caller; the Lua binding is the only plausible one). It is the client's *claim* that the wheel finished; it never chooses the prize. (client, client Lua)
- **Reward preview updates.** **S>C 39/68** (`MiniGameUpdateRewardPacket`) rewrites the state's three reward bundles (at state `+0x90`, `+0x148`, `+0x200`; reward, member reward and preview, matching the three data-source names registered in `009bc410`). 39/22 (round status) also carries two reward bundles for the between-rounds screen. (client)
- **Contents.** Item, coin and star prizes per job are content, not framework (dungeon pages, e.g. [[Robgoblin Treasure Trove]], list six job rows of five items plus a coin slot; [[Dungeons]]). The farm shopping-list games (TYPE 1) say in their own description that filling the order earns "some coin and a spin on the prize wheel". (client locale, video, wiki)
- **Daily wheel.** TYPE 22 row 8 ("Spin For The Win!") is the *Daily Rewards* wheel: "Spin the wheel once a day to win prizes that range from potions to pets to TCG cards!" It is flagged `IS_MICRO`, the client has a "come back tomorrow" message, `RequestWheelInteraction` `009bdbd0` sends **26/11** after checking the scene's wheel field and the local wheel count, and the repeating-activity family (143, count, consecutive and deadline fields) looks like its state. That link is an inference. (client resource, client locale, client; R)

#### Leave, quit, cancel and disconnect

| Path | What the client sends | Notes |
|---|---|---|
| Player quits mid-game | `Ui.QuitMiniGame` (`00c17470` -> `00939100`): **39/6** `[instance, -1, -1]` when the state is not an encounter, otherwise **41/109** | Native Cooking Playing Quit was accepted: the server answered with 39/19, the client's 39/7 ack followed, no 39/18 or 39/47. Offer-time cancel uses 26/42 instead. (native run) |
| Results Close | **26/42** always, **41/109** when the state is an encounter | See the lifecycle table. |
| `CancelMiniGame` `009bef50` | **39/7** | Used when a pending game or invitation is cancelled; also resets the invitation-rejected UI. |
| Movie `Quit`/`endGame` command | same as quit | |
| S>C **39/54** | none | `LeaveMiniGameInstance` `009bf0f0`: marks an eligible state leaving. (likely) |
| Connection lost | nothing | No resume, refund or reward code exists in any exported movie (`GameClientDisconnect` is empty). The recovered server behaviour is a reconstruction (R): cancel the pre-win run on the next world load, consume and award nothing. (client AS2, native run) |

### How individual minigames plug in

| Game | Types | Launch / lobby | In-game protocol | Results | Guide |
|---|---|---|---|---|---|
| Kart Racing | 10 (68 rows), 13 (5), 19, 20 | Gate -> lobby catalogue 102/1, 102/2 and lobby 101, encounter 41/x, then 33/9 | Family 33 (34 subtypes), processor `VehicleRaceProcessor` | `RaceResults:Show` -> score screen page 3; no server finish packet | [[Kart Racing]] |
| Demolition Derby | 11 (48) | Same gate/lobby pattern | `VehicleDemolitionDerbyProcessor` `009f39b0`, same subtype scheme as racing (family 34, `0x22`; handled subtypes are listed in [[Demo Derby]]) | `DerbyResults:Show` -> page 4 | [[Demo Derby]] |
| Soccer | 12 (22; the type row is members only) | Lobby | Family 76 (`00b80ac0`, sub-ops 1 to 25 except 20) | `SoccerResults` -> page 6 and half-time board | [[Soccer]] |
| Fishing | 21 (8) | Encounter | Own proxies and data sources | none; quits to world | [[Fishing]] |
| Pirates | 13 (5) | Ship selection screen sends its own request | Family 94 | `ScoreScreen:Show(status, 13)` | [[Ship Combat (Pirates)]] |
| Chess, Checkers | 6 (12), 7 (6) | Lobby | Chess: movie plus native engine (`ChessLib.dll`, Lua `ChessInit`, `ChessMovePlayer`, `ChessMoveRequest`); Checkers: movie (no native checkers engine tokens found) | pages 6 and 7, score tab hidden | [[Chess]] |
| Trading card game, Treasure War | 16 (124), 23 (1), 3 (row 11) | Lua `StartTutorial` / `StartTreasureWar` call the activity-join sender `00be7d80`, the server creates the state | Separate card client; 39/0x28 carries the launch ticket | page 16 | [[Trading Card Game]] |
| Match-three: Mining, Harvesting, Forging, Mail Sorting, Artifact Hunt | 8 (131) | Crafting stations and nodes; 26/12 launches the movie | Movie protocol over 39/14 | page 8 | [[Miner]], [[Blacksmith]] (job pages) |
| Mimic: Cooking, Smelting, Feeling the Beat, Hack 'n' Slash | 9 (54) | 26/12 launches the movie | Six-phase (cooking) or four-phase (smelting) rows from the server | page 9 | [[Chef]] |
| Tower defence | 14 (46) | 26/12 | Movie | page 14 | none yet |
| Instanced quest games, shopping lists, deliveries, dungeons, battles | 1 (152), 2 (91), 4, 5 (1) | Portal, activity list, group launch or queue | Generic 39 and 45 packets only; the server is the game | page 4 (type 1 and 4) or none (type 2) | [[Dungeons]], [[Combat and Abilities]] |
| Daily wheel | 22 (1) | Interaction 26/11 | Movie `game_wheel` | the wheel itself | this guide |

How a game is *entered* is shared too. The client has five entry routes, all ending in a 39/16 for the same state: (1) the **activity portal** and activity lists (families 147 and 167; the join request `00be7d80` looks the activity up in a 1024-bucket table and sends it through the world tunnel for entry type 1 or the client tunnel for entry type 2); (2) the **lobby** (101 sub-ops 1, 2, 3, 5, 9, with the game catalogue 102/1 and 102/2); (3) **matchmaking queues** (141 and the encounter add-match request); (4) **group launch** (family 175, below); and (5) direct **encounter entry** (41/108 entrance request when client `+0x700` is 1). See [[Matchmaking and Lobby]] for the lobby and queue packets.

#### Group launch (family 175)

`ClientActivityLaunchRequest` objects track an invitation: member list, flags (cancel, launched), and a delegate. `ClientMiniGameActivityLaunchProcessor` (name `Minigame`, `00bb3660`) is the processor that turns the messages into Lua events (`ActivityEvents:OnInviteSent`, `OnInviteResult`, `OnKicked`, `OnMemberKicked`, `OnMemberRemoved`, `OnMatchmakingStart`, `OnMatchmakingLaunch`, `OnMatchmakingCancel`, `OnLaunchCanceled`) and, on launch, joins the multiplayer game (`MiniGameStateManager:joinMultiplayerMiniGame`). The client refuses to interact with the world while an invitation is pending ("Your have been invited to a minigame. Please accept or reject the invitation first!", `Interactions.PendingGaqInvitationWarning`, `0092efe0`). Sub-opcode table (client constructors): 1 InviteDetails, 2 InviteMemberReply, 3 InviteMemberResult, 4 PlayerHasBeenAccepted, 5 ActivityLaunched, 6 ActivityCanceled (dispatcher `009c3590`), 7 AddMember, 8 RemoveMember, 9 UpdateMemberState, 13 KickMemberEvent, 14, 15, 16 matchmaking start, launch, cancel (all S>C); 17 InviteResponse, 18 MemberReady, 19 MemberQuit, 20 InviteMemberRequest, 21 KickMemberRequest, 22 OwnerLaunchRequest, 23 OwnerMatchmakingRequest (all C>S). Sent bodies are one 16-bit value plus three 32-bit values (writer `00c4cd20`), InviteResponse adds an acceptance byte and a decline-reason dword. Bodies are unrecovered. (client)

### Data and formats

**Client resource tables** (caret-delimited, `#` header; counts re-derived from the baseline client's `Resources` folder):

| File | Columns | Contents |
|---|---|---|
| `MiniGameData.txt` | `ID, TYPE, NAME_ID, DESCRIPTION_ID, ICON_ID, DIFFICULTY, MEMBERS_ONLY, SHORTCUT, MIN_PLAYERS, MAX_PLAYERS, START_SOUND_DEFINITION, PRELOAD_FILE, LEADERBOARD_ID` | 783 rows. `MEMBERS_ONLY`, `SHORTCUT`, `START_SOUND_DEFINITION` and `PRELOAD_FILE` are 0 or empty in every row. `MIN/MAX_PLAYERS` are 0/0 except 1/1 on four rows (three spot-the-difference rows and Cannonball Chatty). `DIFFICULTY` is 1 to 5. `LEADERBOARD_ID` is non-zero on exactly 11 rows, all TYPE 8: Copper, Tin, Iron, Silver and Gold Mining (91, 93, 95, 97, 99) and six Harvesting rows (109, 111, 113, 115, 117, 119). 173 rows have no group link. |
| `MiniGameTypeData.txt` | `TYPE, NAME_ID, DESCRIPTION_ID, IS_FLASH, IS_MICRO, PARAM1, HOWTO_SWF_NAME, ICON_ID, MEMBERS_ONLY, BACKGROUND_ID, IS_POPULAR, IS_HIGHLIGHTED, CATEGORY_ID, IS_ACTIVE` | 22 rows (types 1 to 16 and 18 to 23). Names, icons and descriptions are all 0. `IS_FLASH` = 1 on 3, 8, 9, 14, 16, 23 (360 rows); `IS_MICRO` = 1 on 22 only; `MEMBERS_ONLY` = 1 on 12 (Soccer) only; every row `IS_ACTIVE` 1. 417 rows sit on non-flash types and 6 rows on undeclared types (0, 17, 1009, 1011). |
| `MiniGameGroupData.txt` | `ID, NAME_ID, DESCRIPTION_ID, ICON_ID, BACKGROUND_SWF, STAGE_PROGRESSION, SHOW_START_SCREEN_ON_PLAY_NEXT, SETTINGS_ICON_ID, TYPE_ID, PURCHASE_ITEM_ID, PURCHASE_BUNDLE_ID, PURCHASE_PREREQ_ITEM_ID` | 20 groups with ids 1, 2, 3, 5 to 16 and 19 to 23; every other column is 0 or empty. |
| `MiniGameGroupLinks.txt` | `ID, GROUP_ID, PARENT_GAME_ID, LINK_GAME_ID, LINK_NAME, SHORT_NAME, ICON_ID, STAGE_NUMBER, LINK_GROUP_ID` | 610 links. In the shipped file `GROUP_ID` always equals the linked game's TYPE; parent, name, stage and sub-group columns are 0 or empty. A group is therefore "all games of one type" in the data, while the 39/50 group record can carry real stage progression. |
| `GameTutorialData.txt` | `ID, SWF, ICON_ID, STRING_ID` | 18 how-to movies (artifact, farming, forging, mail sorting, mining, cooking, music, smelting, sword, chess, checkers, pet gestures, mahjong, Halloween, soccer, hidden object, tower defence, racing). |
| `ActivityCategories.txt`, `ActivityPortalSections.txt` | `ID, CONTENT_ID, NAME_ID, DESCRIPTION_ID, THUMBNAIL_IMAGE_FILENAME`; `ID, NAME_ID, DESCRIPTION_ID` | 16 categories; 10 portal sections. Used by the portal, the activity lists and the play-more prompt. |

**Known minigame type ids.** Rows are the `TYPE` column of `MiniGameData.txt`; the Lua name is the `MiniGameStatusHandler` constant; results = what `SetMiniGameEndData` does.

| Type | Lua name | Rows | Table flags | What the rows are | Results page | Source |
|---|---|---|---|---|---|---|
| 0 | TUTORIAL | 1 | undeclared | "NOT LIVE" Sacred Glade tutorial | quit, show main | client resource, client Lua |
| 1 | SAMPLE | 152 | non-flash | Quest games: foot race, sheep, shopping lists (40 rows), deliveries, Archer, Ninja and Medic tests, "IWMG" quest games | type 4 page | client resource, client Lua |
| 2 | SCAVENGER_HUNT | 91 | non-flash | Postman delivery games, 60 marked NOT LIVE | none (quits and leaves) | same |
| 3 | FLASH | 4 | **flash** | rows 7, 11 (FR Trading Card Game), 107, 108 | quit, show main | same |
| 4 | COMBAT | 0 | declared, no rows | Dungeons and battles; the id arrives in the info record | type 4 page | client Lua, native run |
| 5 | HIDDEN_OBJECT | 1 | non-flash | Robgoblin Camp hidden-object dig | quit, show main | same |
| 6 | CHESS | 12 | non-flash | Chess at several difficulties | page 6 | same |
| 7 | CHECKERS | 6 | non-flash | Checkers | page 7 | same |
| 8 | MATCH_THREE | 131 | **flash** | Mining (19 rows: five metals, rich variants), Harvesting (22), Forging (62 recipe rows), Mail Sorting (8, six towns), Artifact Hunt | page 8 | same |
| 9 | MIMIC | 54 | **flash** | Cooking (39 dishes), Smelting (5 bars), Feeling the Beat (7 rows over five difficulties), Hack 'n' Slash | page 9 | same |
| 10 | RACING | 68 | non-flash | Kart race rows per speedway, "Vehicle Race In The Wilds" | race page (own call) | same |
| 11 | DEMO_DERBY | 48 | non-flash | Derby rows per arena, tutorials | derby page (own call) | same |
| 12 | SOCCER | 22 | non-flash, members only | Soccer rows per town | soccer page (own call) | same |
| 13 | PIRATES | 5 | non-flash | Shipwreck Cove and unnamed rows | page 13 | same |
| 14 | TOWER_DEFENSE | 46 | **flash** | 12 named maps at easy, medium and hard | page 14 | same |
| 15 | TILE_MATCHING | 1 | non-flash | "DNT" herbalist mahjong | quit, show main | same |
| 16 | TRADING_CARD_GAME | 124 | **flash** | Duel rows, lobby 499, tutorial 537 | page 16 | same, [[Trading Card Game]] |
| 17 | PLATFORMER | 1 | undeclared | one unnamed row | quit, show main | same |
| 18 | (none) | 0 | declared, empty | | | client resource |
| 19, 20 | (none) | 1 each | non-flash | "Wildwood Kart Racing" rows 582, 583 | not handled | client resource |
| 21 | FISHING | 8 | non-flash | Learn to fish and the lagoons | none (quits) | same |
| 22 | WHEEL | 1 | non-flash, **micro** | "Spin For The Win!" daily wheel (row 8) | the wheel | same |
| 23 | (none) | 1 | **flash** | Treasure War (row 727) | not handled in this switch | same |
| 1009 | SPOT_THE_DIFF | 3 | undeclared | Spot the Difference easy, medium, hard (5 differences, 1 player) | page 1009 | same |
| 1011 | (none) | 1 | undeclared | Cannonball Chatty (1 player) | not handled | client resource |

**Info record carried by 39/16** (`MiniGameInfoPacket`; reader `009bfc50` calling `009bdd70`; client reading, field order certain, meanings partly inferred from `00c42c60` and the data-source getter `00c468c0`):

| Order | Field | Use |
|---|---|---|
| header | `u16 39, u8 16`, three `i32` | the first `i32` is the instance id that keys the state. Native Cooking info was `(46000001, -1, 46)`: instance, group `-1`, game id. |
| 1-5 | five `i32` | name string id, icon id, description string id, difficulty, **profile type** (the job profile the game wants; the "Profile Type" column and the job switch message come from here, not from the tables) |
| 6 | `i32` | **game type id** (record `+0x250`) |
| 7 | `u8` | members only (`+0x254`) |
| 8 | three reward bundles | reward, member reward, preview (reader `008e7930`) |
| 9 | `i32` count, then 0xe8-byte records | the goal list: `ObjectiveData` records, each with an embedded reward bundle (reader `009bc380`; `HandleMiniGameStart` turns each into a 0x110-byte goal row through `00c42870`) (likely) |
| 10 | five `u8` | `+0x264` (state `+0x904`), `+0x265` (`+0x905`), `+0x266` (`+0x906`): three HUD flags passed to `HUD:showMinigame`; `+0x267` (`+0x907`): the "interactive start" flag (clear = micro / auto-start); `+0x268` (`+0x908`): **must be set for the results dialog to open** (see the lifecycle table and the vault note on combat results) |
| 11 | `i32` length + bytes | the preload string (asset list file name, e.g. the smelting asset list in the native run) |
| 12 | `i32` `+0x258`, `u8` `+0x269`, `i32` `+0x25c` (becomes state `+4`, the game id), `u8` `+0x26a` (sets manager flag bit 0), `u8` `+0x26b`, `u8` `+0x26c`, `u8` `+0x26d`, `i32` `+0x260` | flags and ids, partly unmapped |
| 13 | `i32` | trailing dword at `+0x298` |

The reader accepts a cursor at or beyond the end of the buffer (it does not enforce exact length). Success sets the 39/16 path; failure of any bounded read zeroes the field and sets an error flag that makes the whole packet ignored. (client)

**Header, as every family-39 packet.** `u16 39, u8 sub-opcode, i32 id1, i32 id2, i32 id3` (reader `009b2340`, base constructor `008a4420`; the 16-bit and 8-bit values are sign-extended into dwords). Ordinary client requests use `[instance, -1, -1]`; server info uses `[instance, -1, game id]`. A 15-byte client request is `27 00 <sub> <i32> ff ff ff ff ff ff ff ff`. The original server's instance-id scheme is unknown; the ids `46000001` and similar in our native runs are our own scheme (R). The tunnelling envelope is the usual `u16 outer | u8 flag | u32 inner length`. (client, native run)

**Score rows (39/47).** After the header: `i32` row count, then per row a length-prefixed string followed by **five `i32`**: icon id, **score type**, count, max, points (reader `009b2b30` writes them at state-row `+0x34..+0x44`; data-source columns Name, Icon ID, Score Type, Score Count, Score Max, Score Points). A short or negative row length sets the error flag. The native combat run showed `scoreEnemiesDefeated [-1, 0, count, -1, 0]` and `scoreTotalScore [-1, 4, -1, -1, points]`. (client, native run)

**Game over (39/18).** After the header: `u8 won`, then three `i32` (state `+0x70/+0x74/+0x78`; native runs use stars, elapsed seconds, total score). (client, native run)

**Other bodies.** 39/50 group info carries a 76-byte record (four `i32`, a string, `i32`, a counted array of 0x70-byte element records, a string, a byte, an `i32`) published as the nine `MiniGameGroup.*` properties. 39/22 carries two reward bundles, two `i32` and an array. 39/45 carries one reward entry. 39/62 carries a `u32` discriminator (1 update: id, type, float; 2 delete: id). 39/1 and 39/2 carry three id arrays / a length-prefixed blob holding three counted sections of game, group and type records (duplicates by id are rejected). 39/23 carries two `i32`. 39/20 carries a float. 39/21 carries an `i32` total and a flag byte. (client)

**Data-source columns** (UI contract, confirmed in S0392):

- `BaseClient.MiniGame` (17): Name, Icon ID, Description, Difficulty, Type, Elapsed Time, Earned Stars, Members Only, Profile Type, Has Action Bar, Total Score, MiniGame Id, IsFlash, IsMicro, HowToSwfName, Param1, LeaderboardId.
- `BaseClient.MiniGameData` (11): Id, Type, Name, Description, Icon ID, Difficulty, Members Only, GroupId, GroupName, GroupIcon, StartSoundId.
- `BaseClient.MiniGameGroup` (32): MiniGameId, Name, Description, Icon, LinkName, ShortName, Unlocked, Allowed, Members Only, ParentMiniGamed (typo preserved in the binary), BackgroundSwf, SelectedMiniGame, StageNumber, MinPlayers, MaxPlayers, StageProgression, ShowStartScreenOnPlayNext, MiniGame Type, Help Swf, Start Sound ID, Leaderboard Position, High Score, Challenges Completed, Challenges Available, Overall Rating, Stage Image, Must Purchase Item ID, PlayedCount, New, Next, IsMicro, Price.
- `BaseClient.MiniGamePortal` (17): TypeId, TypeName, TypeDescription, MiniGameGroupId, IconId, BackgroundIconId, IsMembersOnly, IsMicro, LockStatus, IsActive, CategoryId, CategoryName, HasPlayedBefore, HasNewStages, IsNew, IsGameOfTheDay, IsPopular.
- `BaseClient.MiniGameGoals` (9): ID, Name, Description, Bonus, Status Text, State, Reward Stars, MembersOnly, CategoryPrefix. `BaseClient.MiniGameScore` (6): Name, Icon ID, Score Type, Score Count, Score Max, Score Points. `MiniGame.Bosses` (2): Name, Health. `BaseClient.MiniGame.Trophies` (4): Id, Name, Description, Icon ID.
- `Reward.Entries` / `RewardPreview.Entries` columns: Type, Name, Description, Icon ID, Quantity, Tint Value, Item Id, Parameter 2, Rarity, Members Only, Guid, Hidden (and Show Separated for rewards).

**Flash-movie data rows.** Each movie family has its own server tables, delivered as replies after `FRServer_ReadyForGameData`. The mimic movies (Cooking, Smelting) carry a local test server (`CookingGameServer.as`) whose phase row is 19 caret columns: `ID, GAME, SEQUENCE, DURATION, OBJECTIVE_ID, START_ID, END_ID, STAGE_TYPE, DATA1 .. DATA11`, plus a game row with a bonus score. Our server sends six such rows for cooking and four for smelting; the original rows were never recovered. The match-three movies (Mining, Harvesting, Forging) are driven by build-map, tile-queue, random-queue, quota and powerup messages from a server that owns the board; the original level tables (layouts, quotas, spawn rates, timers) were never recovered either. (client AS2, native run, vault: Original Cooking, Original Forging)

**Leaderboards are not packets.** The in-game leaderboard is a web fetch: `ClientConfig.ini` carries `GameLeaderBoardUrl` (a freerealms.com `ingameleaderboard` action), `ClientLeaderBoardManager::RequestLeaderBoard` throttles at one request per second and `OnGetResult` parses the reply. Only the 11 mining and harvesting rows carry a board id. A separate client command `NetworkCommand_LeaderBoardData` (`0x1ca`) exists. (client, client resource)

### Network

The family is opcode **39** (`0x27`), written by the shared constructor `008a4420`, carried in the client tunnel (gateway id 5, `PacketTunneledClientPacket`). Receive routing is `BaseClient::HandleIncomingPacket` `009471a0` -> `009bfea0` / `009bf890`. The packet catalogue used by the map validator has no family-39 classes, so the map's `wire` field is filled on only 32 functions; every sub-opcode below was read from a constructor, a dispatcher arm or a native run.

**Server to client (39/x)**

| Sub | Class / meaning | Handler | Body | Status |
|---|---|---|---|---|
| 1 | DataLocked: remove game, group, type ids | `009b8c50` | three id arrays | confirmed arm, semantics R |
| 2 | DataUnlocked: install game, group, type records | `009b8f90` | length-prefixed blob, three counted sections | confirmed |
| 14 (`0xe`) | Flash payload | `009b89e0` | counted byte array, tab-delimited string | confirmed |
| 15 (`0xf`) | Message | `009b8630` | `i32` locale string id, `u8` flag | likely |
| 16 (`0x10`) | **Info / JoinGame** | `009bfdb0` -> `009beb70` | info record above | confirmed; native run |
| 17 (`0x11`) | **GameStarted** | `009b9550` | header only | confirmed; native run |
| 18 (`0x12`) | **GameOver** | `009ad480` | `u8` + 3 `i32` | confirmed; native run |
| 19 (`0x13`) | **StateRemove** | `009bf190` | header only | confirmed; native run |
| 20 (`0x14`) | UpdateGameTimeScalar | `00900c20` | float | confirmed |
| 21 (`0x15`) | StarsEarned | `009b8880` | `i32` total, `u8` flag | likely |
| 22 (`0x16`) | RewardStatus (round) | `009ba720` | two bundles, 2 `i32`, array | confirmed arm; body partly unnamed |
| 23 (`0x17`) | KnockOut | `009b87a0` | 2 `i32` | likely |
| 40 (`0x28`) | TradingCardStartGame | `009bf890` | 2 `i32` (launch ticket) | confirmed |
| 41 (`0x29`) | Card client load failure | `009bf890` | none | confirmed |
| 44 (`0x2c`) | StoreAdvertisement | `009b6fe0` | reader `009b7fa0` | confirmed arm |
| 45 (`0x2d`) | **Loot wheel landing item** | `009b9e20` -> `009b6da0` | one reward entry | confirmed arm |
| 47 (`0x2f`) | **GameEndScore** | `009bb030` | score rows | confirmed arm |
| 50 (`0x32`) | GroupInfo | `009bca80` -> `009b9fc0` | 76-byte group record | confirmed |
| 51 (`0x33`) | ShowStageSelect | `009aaef0` | none | confirmed |
| 54 (`0x36`) | LeaveMiniGameInstance | `009bf0f0` | header | likely |
| 62 (`0x3e`) | Boss update / delete | `00c48c90` | `u32` + record | likely |
| 65, 66 (`0x41`, `0x42`) | FirstCompleted / FirstUnlocked events | `009bf890` | header id slot | confirmed arms |
| 67 (`0x43`) | CreateGameResult | `009aa230` | flag byte | arm confirmed, meaning of the flag unproven |
| 68 (`0x44`) | **UpdateReward** | `009b9eb0` | bundles | confirmed arm |

**Client to server (39/x)**

| Sub | Meaning | Sender | Body | Status |
|---|---|---|---|---|
| 3, 4 | CreateGame (two variants) | `009bd590` | `[game, -1, current group]` | likely |
| 5 | **RequestStartGame** | `009be7d0` | `[instance, -1, -1]` | confirmed; native run |
| 6 | **Quit** (pre-win, ordinary game) | `009be8e0` | `[instance, -1, -1]` | confirmed; native run |
| 7 | **Leave / acknowledge** | `009be8f0` (called by `009bf190`, `009bef50`) | `[instance, -1, -1]` | confirmed |
| 8, 9 | SetReady / SetNotReady | `009be910`, `009be970` | `[instance, ...]` | likely |
| 11 | RestartGame | `009bcb60` | `[instance, -1, -1]` | confirmed |
| 14 | Flash payload | `009be6b0` | string with NUL | confirmed; native run |
| 25 (`0x19`), 26 (`0x1a`) | round/stage reports | `009bea10`, `009bea40` | `[instance, value, ...]` | sub 25 seen natively with value 6 (Cooking), 8 (Smelting), 3 (Forging); caller and meaning not traced; 26 never seen |
| 37 (`0x25`) | card client ready | `009bf590` | header | likely |
| 38 (`0x26`) | RequestTCGChallenge | `009bdaf0` | | likely |
| 46 (`0x2e`) | **LootWheelRotationStopped** | `009bda80` | header | likely |
| 48, 49 | CreateGroup / QueryGroup | `009bd450`, `009bd4f0` | `[-1, game, group]` | confirmed |
| 59 (`0x3b`) | ToggleReady | `009be9d0` | `[instance]` | likely |

**Other families the framework uses**

| Fam/sub | Name | Dir | Use |
|---|---|---|---|
| 26/12 | StartFlashMiniGame (`HandleCommandCase12` `00aa0a10`) | S>C | Names the handler (`MiniGameFlash`) and the movie file; `009bd650` opens it, first sending a dismount request. native run |
| 26/11 | InteractionStartWheel | C>S | `RequestWheelInteraction` `009bdbd0` |
| 26/42 | ClosedMinigameEndScreen | C>S | 4 bytes `1a 00 2a 00`, the real results Close. native run |
| 41/107, 108, 109, 114 | Encounter zone ready, entrance request, exit request, details | both | Entry and exit for encounter-state games. 41/109 is `[0, active launch instance]` (the second id comes from client `+0xc10`, set by 41/107). native run, [[Kart Racing]] |
| 45/x | Objectives | both | 1 to 4 and 5 (unhide): minigame goal states, S>C. 6, 10, 11, 12: client-objective registration, S>C. 8 and 9: clear and re-arm a registered objective, S>C. 7: the client's completion report, C>S (`00cffba0`). [[Quests and Objectives]] |
| 95/28 to 64 | `BaseMiniGameAdminPacket` | C>S | Ten request wrappers (`009bcc00` to `009bd1a0`, sub-ops 28, 30, 31, 32, 33, 34, 36, 58, 64) and `MiniGameFlashCommandAdmin` 95/63 (`009bead0`). No UI binding recovered; probably developer tooling. |
| 101/1, 2, 3, 5, 9 | Lobby join, leave, start, create, leave lobby | C>S (world tunnel) | `ClientLobby` virtuals `00bb1890` to `00bb1cd0`. |
| 102/1, 102/2 | Lobby game catalogue | both | See [[Matchmaking and Lobby]]. |
| 141 | Matchmaking queues | both | `Matchmaking` Lua: PopulateQueues, JoinQueue, LeaveQueue, LeaveMatchRequest. |
| 143 | Repeating activity (daily counters) | both | Requests of types 4, 5, 6, 8, 9; responses of types 1, 2, 3, 7, 10. (R: wheel state) |
| 147, 167 | Activity profile / activity lists, scheduled activities, join request | both | Sub-ops 1 and 2 on 147; 1, 2, 5 on 167 (client class constructors). The 147/1 record count and record size are not recovered here (unknown; R if the server must send one). |
| 175 | Activity launch (group) | both | Table above. |
| 33, 34, 76, 94 | Native engine families (racing, derby, soccer, Pirates) | both | Per-game guides. |

### What the Rust server must do

Items marked (R) are reconstructed policy, not original behaviour.

1. **Own the catalogue.** Load the same four tables the client has (`MiniGameData`, `MiniGameTypeData`, `MiniGameGroupData`, `MiniGameGroupLinks`) from the client's `Resources` folder and treat the type id as the behaviour switch. Never trust a client-sent game or type id; resolve it from the server's own state. The tables ship with the client; do not copy them into the repo. Remember that `MEMBERS_ONLY` is set only on the soccer type row.
2. **Mint a unique instance id per run** (the first header id) and never reuse it, including across retries. Keep a ledger per player: one live game at a time, a second start before the first settles is rejected, a stale finish is rejected without packets or rewards. (R, native run)
3. **Launch.** On an accepted entry (portal join, lobby start, queue match, group launch, encounter entrance) send **39/16** with the info record: instance, `-1`, game id in the header; name, icon and description locale ids, difficulty and the **profile type** in the five leading dwords; the type id; goal array; reward and preview bundles; the preload asset-list string for games that need one; and the flag bytes: `+0x267` for micro versus Go click, `+0x26d` to suppress the start screen and **`+0x268` set so the results dialog opens**. Do not send 39/17 or any game message first: the dispatcher ignores them until the state exists.
4. **Accept start with the sentinel header.** The client's 39/5 is `[instance, -1, -1]`, not `[instance, -1, game id]` (a server that insists on the authored game id never answers; this stalled the first native Cooking run). Reply with **39/17** and, for Flash games, **26/12** (`MiniGameFlash` plus the `game_*.gfx` file). Native engines skip 26/12 and run their own entry (see [[Kart Racing]]).
5. **Flash relay.** Answer 39/14 payloads as the movie expects (the per-movie handshakes live in the job and game guides): tolerate the NUL terminator, the trailing `)` on `FRServer_StatUpdate`/`StopLevel` fields, and ignore a reported score or quantity that disagrees with server-owned state. A forged `FRServer_GameWon` must grant nothing; only server-side completion may. (R, native run)
6. **End sequence, exactly in this order:** movie message (`OnGameWonMsg` or equivalent) -> **39/18** `(won, stars, elapsed, score)` -> **39/47** with at least one real row (always a type-4 `scoreTotalScore` row with the true total) -> **39/19**. Without any reward, stat or score page the client closes the screen and exits on its own, and an empty total row shows "0". Send reward and preview data (39/68 and the bundles) before the screen opens. (R for the 39/68 timing; native run for the rest)
7. **Ignore the client's automatic 39/7 after 39/19** and keep the run in its terminal state until the real Close (**26/42**, or **41/109 `[0, instance]`** for encounters). Apply the economy on win, once. Close must not send a second 39/19. (native run)
8. **Quit and disconnect.** Accept pre-win **39/6** for the current instance and answer with a 39/19 only: no 39/18, no 39/47, no consumption, no reward (R). On disconnect or world load, cancel any pre-win run without consuming or granting (R, native run). Reject malformed, stale or foreign instance ids without mutation.
9. **Rewards are server commits.** Consume inputs, grant outputs, advance quests and record the completion in one transaction; persist before replying. Use content-defined amounts; the vault's 10 bars, 1 dish and 1 hammer are reconstructed rules, not original values. (R)
10. **Prize wheel.** Choose the winning entry on the server, send the preview data, then **39/45** with the winning entry *before* the player spins; accept the **39/46** stop report only for the current instance and pay at most once per instance. A wheel without entries is skipped by the client. Slot contents are content: coin amount per dungeon, "Battle Item Mystery Pack" and per-job gear. (R for the exact ordering; client for the display behaviour)
11. **Stars and HUD.** Send 39/21 when stars rise (the client shows only the increase), 39/23 for knockouts, 39/20 for time scaling if a game needs it, and 45/1 to 4 for goal state while a state exists. The star total in the results page comes from the 39/18 dwords. (client)
12. **Group, lobby and queue entry.** Serve 39/48 and 39/49 with 39/50 group records if stage select is to be used; answer 39/3 and 39/4 with 39/67; serve the lobby catalogue and the queue and group-launch families from [[Matchmaking and Lobby]] (the 175 bodies are unrecovered). Serve the activity lists (147/167) with a schedule if scheduled activities should appear.
13. **Definition pushes are optional.** 39/2 and 39/1 can add or remove game, group and type records at runtime; the shipped tables already cover the retail catalogue.
14. **Leaderboards.** Either serve the web endpoint named by `GameLeaderBoardUrl` or leave it unreachable; never accept a client-computed score as a leaderboard entry. (R)
15. **Per-game rules.** Everything inside a game (level tables, recipes, rewards, balance) is outside the framework and mostly unrecovered; see the per-game guides.

### Evidence

- **Client functions.** 499 accepted functions across the 34 listed shards: 173 confirmed, 198 likely, 56 speculative, 72 unknown; 32 carry a `wire` link. The framework core is S0269 to S0274 (packet constructors and the manager `009bc410`, S0272 and S0273 the dispatchers), with S0392 and S0393 (data sources, goals, bosses, activity launch), S0366 and S0376 (activity processors and join), S0245 and S0247 (leave, cancel, interaction warning, loading barrier), S0383 (Lua module tables), S0324 (repeating activity), S0422, S0425, S0426 and S0581 (activity data sources and definitions). The same range also holds unrelated code that the generated table lists under these subsystems: the Station Cash store client, the objective family, Pirates ship selection, soccer packets and the card-game managers.
- **Client Lua and Flash.** `ScriptsBase.lua` (`MiniGameHandler`, `MiniGameStatusHandler`, `MiniGameFlash`, `ScoreScreen`, `RaceResults`, `DerbyResults`, `SoccerResults`), `score-screen/scripts/frame_1/DoAction.as` and the `scorescreen` package, and `SoeGameFramework.as`, `SoeInterfaceFreeRealms.as` and `SoeNetworkTypeFreeRealms.as` from the exported crafting and gathering movies (`SoeGameFramework.as` is byte-identical across Cooking, Smelting, Forging, Mining and Harvesting, per the vault).
- **Client resources.** `MiniGameData.txt`, `MiniGameTypeData.txt`, `MiniGameGroupData.txt`, `MiniGameGroupLinks.txt`, `GameTutorialData.txt`, `ActivityCategories.txt`, `ActivityPortalSections.txt`, `ClientConfig.ini` (`GameLeaderBoardUrl`), and the locale export for names and descriptions.
- **Captures.** OSFR-held packet captures are not used; no recorded original-server traffic backs any statement on this page. The vault pieces below contribute client facts only.
- **Native runs.** Cooking 164218, 171251, 054421 and the Playing Quit 005748; Smelting 173814 and the second smelt/forge in 103605; Forging 181715 with the cancellation restart 181038; cave combat results 154226, 114509, 121834; Mining 18 partial. Reviews: Native Cooking Finish and Mae 171251, Native Gear Up and Smelting 173814, Native Forging Victory 181715, Native Crafting Quit 005748, Native Opening 17 103605, Native Mining Score 121834. Piece 015 receipts bind rows 46, 77 and 139 as the three natively accepted rows; the other 780 are not.
- **Vault notes** (client facts only): Native Combat Results Lifecycle, Native Crafting Results Lifecycle, Native Crafting Results Close, Ordinary Crafting Quit Acceptance, Cave Results Score Binding, Native Loss Boss Display, Pieces 015, 016, 017, 019, 020 and 063, and the original-movie notes for Cooking, Smelting and Forging. Their community-reference comparisons were not used.
- **Video.** Results screens ("YOU WIN!", recipe or dungeon subtitle, TOTAL SCORE, stats lines, "Your Rewards" and "Score Info" panels, Coins and Stars lines, a "Members Only Bonus!" block) for dungeons and crafting; eight-slot prize wheels after dungeons, Artifact Hunt and shopping-list games; race results tables; pre-game panels with Difficulty, Prizes and Go. One dungeon run shows Stars x103, Coins x90 and a members bonus of 105 coins; one Chef win shows Stars x53 and Coins x452 (that panel may include other bonuses). See [[Robgoblin Treasure Trove]] and [[Chef]].
- **Wiki.** Used only for content (prize-wheel item names, difficulty tiers), never for packets.

### Open questions and contradictions

- **No original win/loss traffic is available to this project.** 39/16, 39/17, 39/18, 39/19, 39/22, 39/45, 39/47, 39/68 and 26/42 bodies are known from the client readers and our native runs only. Reward amounts, wheel odds, star formulas, level tables and which reward packet the original sent when are unknown.
- **`IS_FLASH` disagrees with the client's own predicate.** The type table flags 3, 8, 9, 14, 16, 23; `IsMiniGameFlashType` also accepts 6, 7, 15, 22 and 1009. Types 16 and 23 are flagged Flash although the card clients are native/DLL; the real route for each type is the code, not the flag.
- **Type 4, 1003 and 1011.** Type 4 (combat) has no `MiniGameData` row yet drives the dungeon result page; the type reaches the client in the info record, which the server must supply. State type `0x3eb` (1003) has special removal handling and no data row. Type 1011 (Cannonball Chatty) and type 5 (hidden objects) have no recovered client implementation.
- **Which packet closes a kart, derby or soccer run in the original.** Native crafting shows the Close as 26/42 with 39/7 as the automatic ack of 39/19; combat uses 41/109. What the original used for racing, derby and soccer (39/19, or 39/7 through the leave path) is unknown. See [[Kart Racing]].
- **39/25 and 39/26.** Sent by `009bea10` and `009bea40` with a second id; native Cooking sends `[instance, 6, -1]` once per round, Smelting 8 and Forging 3. The caller and meaning are unknown.
- **39/22 round status** body fields (the two `i32`, the array) and **39/67** (what the leading byte means) are unnamed. 39/44 reader and 39/54 use are only partly read.
- **The info record.** Fields `+0x258`, `+0x260`, `+0x269`, `+0x26a` (sets manager flag bit 0), `+0x26b`, `+0x26c` and the trailing dword `+0x298` are not named; the string is assumed to be the preload list (consistent with the native Smelting info "preloads the existing `SmeltingAssets.txt`"). The roles of `+0x267` (interactive start), `+0x268` (results dialog) and `+0x26d` (suppress start screen) come from reading `009beb70`, `009bf5e0`, `009bd650` and `009b7170`, not from a test of each value.
- **Wheel details.** The matching between 39/45 and the preview index is a pseudocode reading of `009b6da0`; the exact entry identity it compares is unproven. Whether the original sent 39/68 before or after 39/18 is unknown. The daily wheel's state packets (143 versus 26/11) are inferred.
- **Group data.** `MiniGameGroupData.txt` names, stages and purchases are all empty, while the 39/50 record and the 32-column group source can carry stage progression, unlock state, price and ratings. Where retail groups were defined is unknown.
- **Leaderboards.** Only 11 rows have board ids. The service behind `GameLeaderBoardUrl` is gone; direction, ties and rewards are unrecovered.
- **Admin family 95** (`BaseMiniGameAdminPacket` and `MiniGameFlashCommandAdmin`) has ten request wrappers and no known UI path.
- **Group launch 175** has no known bodies; sub-opcode 6's constructor was not found in the shard (it is routed by the dispatcher).
- **Earlier notes corrected here.** The Items guide's remark that the prize-wheel stop packet's sender was not traced is closed to a likely: constructor `009aa0c0` (sub 46) and wrapper `009bda80`, still with no recorded caller. Piece 016 lists 39/7 and 39/11 as unnamed; the client labels are Leave/ack (`009be8f0`) and RestartGame (`009bcb60`). 39/23 is a minigame knockout message, not a vehicle-race 33/23.
- **Names.** Several shopping-list rows are duplicated in two blocks of 20; TYPE 3 rows 7, 107 and 108 have empty or unrelated text.

### Key functions

| Address | Role |
|---|---|
| `009bc410` | `ClientMiniGameManager` constructor (loads the four tables, registers data sources) |
| `009bfea0` | `DispatchMiniGameMessage`, main family-39 dispatcher (confirmed) |
| `009bf890` | `DispatchMiniGamePacket`, fallback dispatcher: 14, 17, 18, 19, 22, 40, 41, 47, 54, 65, 66 (confirmed) |
| `009bfdb0` / `009beb70` | `HandleMiniGameJoinGame` and `HandleMiniGameStart`: create the state from 39/16 (confirmed) |
| `009bf5e0` | `TickMiniGameManager`: loading state machine, start countdown, auto-start (confirmed) |
| `009be7d0` | `RequestStartMiniGame`: 39/5 (confirmed) |
| `009bdc60` | `SendMiniGamePacket`: shared family-39 sender, resolves instance -1 (likely) |
| `009b9550` / `009b9500` | `HandleMiniGameGameStarted` (39/17) and `MiniGameState_StartActive` (called on 41/2 for encounter states) |
| `009ad480` | `ApplyMiniGameGameOverResult`: 39/18, stores the result (confirmed) |
| `009bb030` | 39/47 score rows into the state |
| `009bf190` | `LeaveMiniGame`: 39/19, end dialog, automatic 39/7 (confirmed) |
| `009b7170` | `ShowMiniGameEndDialog`: hands the end data to Lua |
| `009bd240` / `008d2730` | `HandleMiniGameLeave` / `LeaveMiniGameOrEncounter`: Close path (41/109, 26/42) |
| `00939100` | `Ui.QuitMiniGame` path: 39/6 or 41/109 |
| `0092f300` | `SendClosedMinigameEndScreen`: 26/42 (speculative label) |
| `009bef50` | `CancelMiniGame`: pending cancel, 39/7 (confirmed) |
| `009bd650` | `StartFlashMiniGameUi`: opens the movie (confirmed) |
| `009be6b0` | `SendMiniGameFlashPayload`: 39/14 (confirmed) |
| `009b89e0` | Flash payload receive -> `MiniGameFlash:OnMessage` |
| `009b6aa0` | `ShowMiniGameHud`: per-type HUD |
| `009b6da0` | `StopMiniGameLootWheelAt`: wheel landing index (confirmed) |
| `009bda80` | Loot-wheel rotation-stopped sender, 39/46 (likely) |
| `009b9eb0` | 39/68 UpdateReward arm |
| `009bca80` / `009b9fc0` | 39/50 group info and the nine group properties |
| `009b62a0` | `LoadMiniGameDataTables`: the four tables (likely) |
| `008a5850` | `IsMiniGameFlashType`: key routing to the movie (confirmed) |
| `00c42c60` / `00c42870` | Build the state's goal list from the info record |
| `00c468c0` | `BaseClient.MiniGame` value getter (columns incl. Profile Type) |
| `00c43010` / `009be130` | Objective runtime and the family-45 dispatcher |
| `00c48c90` | Boss update / delete (39/62) |
| `00bb3660` | `ClientMiniGameActivityLaunchProcessor` constructor |
| `00c4e490` | Family-175 receive dispatcher |
| `00be7d80` | Activity join request sender |
| `00936d00` | `WaitForWorldReady`: loading barrier incl. native minigame processors |
| `00c08480` / `00c082d0` | Lua `SendMessage` / `LoadData` for Flash movies |
<!-- NARRATIVE END -->

## Mapped functions (generated)

499 accepted functions: 173 confirmed, 198 likely, 56 speculative, 72 unknown. Network-linked (`wire`): 32.

Subsystems: `game.activity` (170), `ui.minigame` (86), `game.minigame` (67), `net.minigame` (51), `minigame` (40), `game.repeating_activity` (26), `minigame.activity` (25), `ui.activity` (21), `net.minigame.soccer` (10), `minigame.draft` (2), `minigame.derby` (1)

### `game.activity`

**ActivityDataSourceMap** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cea3a0` | ActivityDataSourceMap::FindOrCreate | speculative | Looks up a datasource by hash and key; returns it if matched, returns null when creation is disabled, otherwise allocates a datasource, inserts it, and acquire… |
| `00cea4c0` | ScheduledActivityDataSourceMap::FindOrCreate | speculative | Looks up scheduled datasource by hash and key; returns it if found, otherwise optionally allocates, inserts, and references a new scheduled datasource. |

**ActivityLaunchRequest** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010868f0` | ActivityLaunchRequest::DtorBody | confirmed | Fastcall dtor body (EH variant), ECX = this. Stores ActivityLaunchRequest::vftable at +0x0, sets the Array<unsigned char,0,1> vftable at +0x48, frees the buffe… |
| `01086970` | ActivityLaunchRequest::ScalarDeletingDtor | confirmed | Thiscall with one byte flag arg, ret 4. Calls 010868f0 (ActivityLaunchRequest dtor body) on this, then if bit 0 of the flag is set calls the free thunk 00d8246… |
| `01086990` | ActivityLaunchRequest::FormatDescription | confirmed | Thiscall with one stack arg (output string), no return value (ret 4). Appends 'RequestorGuid=%llu, SysHashkey=%d, SysDefId=%d, MembersOnly=%d, ReqId=%d, MinMem… |
| `01086840` | — | unknown | Fastcall default ctor, ECX = this, no stack args. Calls 00d0a4e0 (LoggingHeader base constructor) on this, stores ActivityLaunchRequest::vftable at +0x0, sets … |

**ClientActivityDefinition** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01082860` | ClientActivityDefinitionDtorBody | confirmed | Destructor body of ClientActivityDefinition: stores the ClientActivityDefinition vftable, runs the FeaturedActivityEntry list cleanup 010827f0, then releases t… |
| `01082c70` | ClientActivityDefinition::ClientActivityDefinition | confirmed | Fastcall default constructor (ECX = this, no stack args). Stores ClientActivityDefinition::vftable at +0x0, sets two empty SoeUtil::IString<char> members at +0… |
| `01082d30` | ClientActivityDefinition::~ClientActivityDefinition | confirmed | Scalar deleting destructor (ClientActivityDefinition::vftable slot 0): calls the destructor body 01082860, then if bit 0 of its flag argument is set calls the … |
| `01082d50` | ClientActivityDefinition::ClientActivityDefinition | confirmed | Thiscall constructor with one stack arg, stored at this+0x4. Stores ClientActivityDefinition::vftable at +0x0, sets the same two empty IString<char> members at… |

**ClientActivityLaunchManager::ClientActivityLaunchProcessorStub** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009c2230` | ClientActivityLaunchProcessorStubCtor | confirmed | Constructs the base object, installs the processor-stub vtable, stores the supplied key at +0x3c, and assigns the supplied name or an empty default string. |
| `009c22d0` | ClientActivityLaunchProcessorStubScalarDeletingDtor | confirmed | Destroys the processor member and frees the object when delete flag bit 0 is set. |

**ClientActivityLaunchRequest** (29)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c4d730` | ClientActivityLaunchRequest::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): runs the request destructor 00c4d370 and frees the object with CRT _free when bit 0 of the flag is set. |
| `00c4e1b0` | SendReadyNotification | confirmed | Sends the readiness notification: constructs MemberReady (00c498a0) with the request header words, serialises and sends it through 00c4dc60 and logs 'Sent Read… |
| `00c4e490` | HandleActivityLaunchMessage 🌐 | confirmed | Dispatcher for received family-0xaf launch messages: constructs a BasePacket-derived local object and reads the family header with 009c1e00, then switches on t… |
| `00c4b700` | ClientActivityLaunchRequest_BuildDisplayName | likely | Rebuilds the request's display/log name: constructs a temporary string, formats 'ActivityLaunchRequest(%d) - ' with the request id from this+0xa4 through the S… |
| `00c4b7a0` | ActivityLaunchRequest_DumpMemberList | likely | Walks the member chain from the head at this+0x98 through the +0x50 links and appends one line per member to the caller's string: the format '%llu,%s,%d,%d,%d,… |
| `00c4c070` | ActivityLaunchRequest_AddMember | likely | Applies an add-member record: scans the member chain from +0x98 for the incoming member id; when the member exists it logs 'AddMember for an existing member, u… |
| `00c4c1a0` | ActivityLaunchRequest_RemoveMember | likely | Removes a member by id: walks the member chain from +0x98 comparing the id at node+0; on a hit it checks whether the member's guid equals the local player's gu… |
| `00c4c300` | HandleInviteMemberReplyMessage | likely | Handles the invite-member-reply message: constructs the InviteMemberReply packet 00c4b1a0 with the request's header words, parses the body with 00c4acd0 and, w… |
| `00c4c440` | HandleInviteMemberResultMessage | likely | Handles the invite-member-result message: constructs the InviteMemberResult packet 00c4b250, parses the body with 00c4adb0 and, on success, logs 'Received Invi… |
| `00c4c7f0` | HandleKickEventMessage | likely | Handles the kick-event message: constructs the two-member KickMemberEvent packet 00c4b510, parses it with 00c4af20 and, on success, logs 'Received KickEvent. K… |
| `00c4ca90` | HandleOnMatchmakingLaunchMessage | likely | Builds the OnMatchmakingLaunch packet object, reads the shared header from the incoming buffer, logs 'Received OnMatchmakingLaunch.' when parsing succeeds, the… |
| `00c4cb60` | HandleOnMatchmakingCancelMessage | likely | Handles the matchmaking-cancel message: constructs the OnMatchmakingCancel packet 00c49d20, parses the header with 009c1e00 and, when the parse succeeded, logs… |
| `00c4ce40` | HandleUpdateMemberStateMessage | likely | Handles the member-state update: constructs the UpdateMemberState packet 00c4aa70, parses the id and state byte with 00c4b9e0, searches the member chain from +… |
| `00c4d370` | ClientActivityLaunchRequest::~ClientActivityLaunchRequest | likely | Destructor: installs the ClientActivityLaunchRequest vtable and the embedded member-list vtable, drains the member list (loop of 00c4bb70 on the head at +0x98)… |
| `00c4d4a0` | HandleAddMemberMessage | likely | Handles the add-member message: constructs the AddMember packet 00c4b3e0 with the request header words, reads the member fields with 009c24e0, applies them thr… |
| `00c4d840` | ApplyInviteDetails | likely | Applies an invite-details buffer: constructs the InviteDetails packet 00c4d590 (passing the request's header words, the request object and its embedded list at… |
| `00c4d980` | HandleInviteDetailsMessage | likely | Handles the invite-details message: applies it with 00c4d840 and, when that succeeded but the local player's guid (from (+0xac)->+0xa4, +0x3a0/+0x3a4) is not f… |
| `00c4dfe0` | SendInviteResponse | likely | Sends the player's answer to an invite: constructs ClientActivityLaunchPacketInviteResponse (00c49610) with the request's header words from +0xa8/+0xa4, stores… |
| `00c4e090` | SendOwnerLaunchRequest | likely | Sends the launch request: constructs OwnerLaunchRequest (00c49a20) with the request header words, serialises and sends it through 00c4dac0 and logs 'Sent Owner… |
| `00c4e120` | SendOwnerMatchmakingRequest | likely | Sends the matchmaking request: constructs OwnerMatchmakingRequest (00c49ae0) with the request header words, serialises and sends it through 00c4db90 and logs '… |
| `00c4e240` | SendQuitNotification | likely | Sends the quit notification: constructs MemberQuit (00c49960) with the request header words, serialises and sends it through 00c4dd30 and logs 'Sent QuitNotifi… |
| `00c4e2d0` | SendPlayerInvite | likely | Sends a player invite: constructs InviteMemberRequest (00c4b100) with the request header words, stores the target guid at +0x18/+0x1c and the name through SoeU… |
| `00c4e3b0` | SendPlayerRemove | likely | Sends a player removal: constructs KickMemberRequest (00c4b470) with the request header words, stores the target guid at +0x18/+0x1c and the name through 00402… |
| `00c4a550` | — | unknown | Walks the member chain from the head at this+0x98 through the +0x50 next links and returns the first node whose guid (+8/+0xc) matches the requested pair, or, … |
| `00c4c240` | — | unknown | Constructs a local ClientActivityLaunchPacketPlayerHasBeenAccepted from the request header fields at +0xa8/+0xa4, reads the incoming buffer with 009c1e00, and … |
| `00c4c580` | — | unknown | Handles the activity-launched message: constructs the ActivityLaunched packet 00c4b300, parses the header 009c1e00 and the participant guid list 00c4a810, reso… |
| `00c4c6e0` | — | unknown | Handles a received remove-member message: constructs the packet (00c49790), reads the two fields with 00c4aeb0, searches the member chain from +0x98 for the id… |
| `00c4c9c0` | — | unknown | Handles the matchmaking-start message: constructs the packet object 00c49ba0 with the request's header words, parses the buffer with the shared header reader 0… |
| `00c4d2a0` | — | unknown | Constructor of the client-side launch request: runs a base initialiser 01086840, installs ClientActivityLaunchRequest::vftable, initialises the local/owner gui… |

**ClientActivityManager** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00be6700` | ClientActivityManager::ClearLists | confirmed | Calls the no-argument thunks 00ce9190 and 00ce91a0 (each loads this+0x428 into ECX and tail-jumps to 00cfbe20), then pops and frees every node of the list at t… |
| `00be6d30` | ClientActivityManager::LoadActivityFiles | confirmed | Calls 00725ee0 on its first argument, appends '/' (006fc030 with 0x2f) when the last character of the path buffer local_120 is not '\' or '/', then calls the t… |
| `00be70b0` | ClientActivityManager::ClientActivityManager | confirmed | Stores BaseDefinitionManager then ClientActivityManager vtables at *this; constructs the member hash lists through 00be5e60, 00be5ec0, 00be6e10 and 00be5f20; c… |
| `00be71e0` | ClientActivityManager_ScalarDtor | likely | Calls 00be6bb0 (ClientActivityManager destructor body) on this, then calls 00401c00(this, 0) (FreeMemory) when bit 0 of the flag argument is set; returns this. |
| `00be6bb0` | — | unknown | Sets an EH frame and stores the ClientActivityManager vtable at *this; calls 006fc440 and, when it returns nonzero, 006fd410('ActivityUserData.ini', 0); drains… |

**ClientRequirementCheckCountry** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ce7570` | ClientRequirementCheckCountry::LoadValue | speculative | Calls conversion helper on field +0x30 and stores its return at +0x60. |
| `00ce7860` | ClientRequirementCheckCountry::Evaluate | speculative | Reads country value from client context and compares it with loaded requirement value using the configured integer operator. |

**ClientRequirementCheckCurrentProfile** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ce78f0` | ClientRequirementCheckCurrentProfile::Evaluate | speculative | Reads current profile selector and compares it against the stored value with configured integer operator. |

**ClientRequirementCheckLanguage** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ce7890` | ClientRequirementCheckLanguage::Evaluate | speculative | Reads language value from client context and compares it with loaded requirement using configured integer operator. |
| `00ce7a90` | ClientRequirementCheckLanguage::LoadValue | speculative | Copies string at object +0x34, passes it to a conversion helper, and stores its result at +0x60. |

**ClientRequirementCheckLastPlayedProfileOfTypeRank** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ce7980` | ClientRequirementCheckLastPlayedProfileOfTypeRank::Evaluate | speculative | Finds profile entry keyed by stored type, resolves it through client map, and compares its rank with configured value. |

**ClientRequirementCheckNudge** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ce79f0` | — | unknown | Scans context list for matching type/id fields, then returns match or its negation according to selector +0x1c. |

**ClientRequirementCheckRenamePending** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ce7920` | ClientRequirementCheckRenamePending::Evaluate | speculative | Checks profile API value 3 and applies selector 1 as equal-to-3 or selector 2 as not-equal-to-3. |

**ClientRequirementCheckVipRank** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ce78c0` | ClientRequirementCheckVipRank::Evaluate | speculative | Reads VIP rank from client context and compares it with stored float requirement using the configured operator. |

**ClientScheduledActivityDefinition** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010830f0` | ClientScheduledActivityDefinition::DtorBody | confirmed | Fastcall, ECX = this. Stores ClientScheduledActivityDefinition::vftable at +0x0 and returns (no member cleanup in this body; 7 bytes: mov [ecx], imm32; ret). |
| `01083260` | ClientScheduledActivityDefinition::ClientScheduledActivityD… | confirmed | Fastcall default constructor (ECX = this, no stack args, plain ret). Stores ClientScheduledActivityDefinition::vftable at +0x0, sets dwords +0x1c, +0x20, +0x38… |
| `010832c0` | ClientScheduledActivityDefinition::~ClientScheduledActivity… | confirmed | Thiscall with one byte flag argument. Stores ClientScheduledActivityDefinition::vftable at +0x0; if bit 0 of the flag is set, calls the free thunk 00d82463 on … |
| `010832e0` | ClientScheduledActivityDefinition::ClientScheduledActivityD… | confirmed | Thiscall constructor with one stack dword stored at +0x8 (the record Id, per the dump function 01083590). Stores ClientScheduledActivityDefinition::vftable at … |

**ClientTimerEvent** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009c52b0` | ClientTimerEventCtor | confirmed | Constructs the ExecuteLuaEvent base and installs ClientTimerEvent's vtable, then clears its timer-specific flag byte. |
| `009c5330` | ClientTimerEventScalarDeletingDtor | confirmed | Calls the timer-event teardown routine, conditionally frees the object when the deleting flag is set, and otherwise returns its pointer. |

**SoeUtil::HashList<ClientActivityDefinition::FeaturedActivityEntry,16,-1>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01082570` | HashListActivitySlot1AllocNode | likely | Virtual slot 1: calls 006fb9c0 once, then 006fb860 for the thread manager; if present calls its vslot 1; else the global manager vslot 1; else calls 006fb950(0… |
| `010825b0` | HashListActivitySlot2FreeNode | likely | Virtual slot 2, stdcall-style (ret 4): pushes alignment 8 and the pointer, calls 00401c00 (FreeMemory). Bytes not decoded beyond the pseudocode; the 18-byte si… |
| `01082920` | HashListActivityCtor16 | likely | Constructor body: stores the HashListMap<int,ClientActivityDefinition::FeaturedActivityEntry,16,-1> vftable, zeroes +0x10, +0x8, +0xc and +0x4, clears the 16-b… |
| `010829a0` | HashListActivityScalarDeletingDtor | likely | Virtual slot 0 scalar deleting destructor: calls the destructor body 010827f0 on this, then frees this with _free when flag bit 0 of the argument is set, and r… |

**SoeUtil::HashListMap<int,ClientActivityDefinition::FeaturedActivityEntry,16,-1>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01082450` | HashListMapActivityCtor16 | likely | Constructor body: stores the HashListMap<int,ClientActivityDefinition::FeaturedActivityEntry,16,-1> vftable, zeroes +0x10, +0x8 and +0xc and +0x4, and clears t… |
| `01082780` | HashListMapActivityDtorBody | likely | Destructor body: stores the HashListMap<int,ClientActivityDefinition::FeaturedActivityEntry,16,-1> vftable and removes list nodes through 00be0a00 until the he… |
| `010827b0` | HashListMapActivityScalarDeletingDtor | likely | Virtual slot 0 scalar deleting destructor: stores the vftable, removes list nodes through 00be0a00 until the head is null, then frees this with _free when flag… |
| `010827f0` | HashListMapActivityDtorBodyEh | likely | Destructor body (EH variant): stores the HashListMap<int,ClientActivityDefinition::FeaturedActivityEntry,16,-1> vftable and removes list nodes through 00be0a00… |

**(no class)** (101)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009c19a0` | AcceptActivityLaunchInvite | confirmed | Walks from list head +0x98 through +0xb8 links looking for id +0xa4; on a match, passes the node in ECX to 00c4dfe0 with acceptance 1 and decline reason 0, the… |
| `009c1a00` | RemoveActivityLaunchRequest | confirmed | Logs removal, removes request id from a secondary map, optionally notifies a processor, unlinks the request from the list, and frees it. |
| `009c1c60` | DeclineActivityLaunchInvite | confirmed | Walks the request list for the supplied id; on a match calls 00c4dfe0 with the matched node, acceptance 0, and decline reason 3, then logs and removes the requ… |
| `009c22c0` | GetField08 | confirmed | Loads and returns the 32-bit value at object offset +8; bytes are mov eax,[ecx+8]; ret. |
| `009c2920` | RegisterActivityLaunchProcessor | confirmed | Gets processor name, hashes it into 32 buckets, logs registration, removes an existing same-key entry, and inserts the new processor. |
| `009c2cc0` | ActivityLaunchManagerDtor | confirmed | Rewrites the embedded list vtable, releases two shared references, repeatedly removes remaining request nodes, then runs list cleanup. |
| `009c2fc0` | ActivityLaunchManagerInit | confirmed | Initializes a request list and processor map, stores the manager argument, then creates and retains two named launch request/member manager objects. |
| `009c3270` | HandleActivityLaunchRequest | confirmed | Parses launch-request fields; on the merge path logs and removes the prior keyed request, checks current launch identity/state, looks up the matching processor… |
| `009c4c30` | FormatTimerDefinitions | confirmed | Writes a timer-definition heading and formats each node's name, duration, flags, callback, and four parameters into the configured output sink. |
| `009c4d70` | FormatActiveTimerNames | confirmed | Writes an active-timer heading, traverses active timer nodes and formats each node name to the diagnostic sink. |
| `009c5990` | LoadTimerDefinitions | confirmed | Clears existing definitions, opens ClientTimerDefinitions.txt, skips comments, splits records on caret and comma delimiters, parses fields, and inserts definit… |
| `009c5cd0` | LoadSavedClientTimers | confirmed | Clears two saved-record maps, reads SavedClientTimers.txt, parses caret-delimited rows and two client-identifier fields; both identifier-comparison branches ca… |
| `009c6670` | SaveClientTimers | confirmed | Collects persistent timer records, serializes timer names, client identifiers, elapsed values and flags to SavedClientTimers.txt, then clears the saved map. |
| `00be6760` | Tutorial_SetOrClearEntry | confirmed | Formats 'Character.%llu' from the 64-bit pair at (*(this+0x18cbc))+0x3a0/+0x3a4 and 'Tutorial.%d' from param_2, calls 00910930 to format/store with param_3, th… |
| `00be68c0` | ActivityUserData_LoadTutorialIds | confirmed | Calls 006fc440 and, if it returns nonzero, 006fd410('ActivityUserData.ini', 0); calls 006fef80 and 006feeb0('ActivityUserData.ini'), clears the table at this+0… |
| `009c1060` | ClientActivityLaunchRequestCtor | likely | Calls 00c4d2a0 with the supplied pointed-to value and returns the first argument unchanged. |
| `009c15e0` | ClientActivityLaunchRequestScalarDeletingDtor | likely | Calls 00c4d370 to destroy the request and frees this when delete flag bit 0 is set. |
| `009c17c0` | FindActivityLaunchRequestById | likely | Walks forward from list head at +0x98 via next pointer +0xb8 and returns the first node whose id at +0xa4 matches, or zero at end. |
| `009c1860` | UnlinkAndDestroyActivityLaunchRequest | likely | Repairs both request-list link fields and head/tail slots, decrements the list count, calls 00c4d370 with node param_2 in ECX, then invokes the list free hook … |
| `009c1c20` | RemoveNodesWithByteB4Set | likely | Walks the request list and removes each node whose byte at +0xb4 is nonzero, preserving the next pointer before unlinking. |
| `009c21b0` | InitializeActivityLaunchPacketMember | likely | Clears the first field, copies two constants into fields +8/+0x0c, constructs an embedded object at this+0x10, clears bytes +0x44/+0x45, and returns this. |
| `009c2350` | ConstructActivityLaunchProcessorStub | likely | Calls 009c2230 with *param_3 and *param_4, stores *param_2 at this+0x48, and returns this. |
| `009c4100` | FindTimerDefinitionByKey | likely | Hashes the key field, walks its bucket chain, compares candidate keys, and returns the matching definition or null. |
| `009c4170` | FindSavedTimerByKey | likely | Hashes a key, traverses its bucket chain, compares keys and then invokes a string comparison before returning a matching node or null. |
| `009c41f0` | FindActiveTimerByString | likely | Walks nodes linked at offset 0x14, compares each node using a string comparison helper, and returns the first match or null. |
| `009c4230` | ReadInt64FromDelimitedField | likely | Copies a delimited field into a 2048-byte local buffer, parses it as signed 64-bit decimal, and writes the parsed result to the output. |
| `009c42a0` | ReadInt64FromDelimitedFieldAlt | likely | Copies a delimited field into a 2048-byte local buffer and parses it with the local signed decimal conversion routine. |
| `009c4630` | SixIStringsInit | likely | Initializes six embedded IString objects to the shared empty buffer, then clears bytes at offsets +0x14, +0x15, and +0x16 of the first member. |
| `009c4820` | InitializeSixStringTimerNode | likely | Initializes six embedded IString fields on the object passed in ECX, stores the supplied dword at object offset 0x70, and returns this. |
| `009c48e0` | InitializeTimerStringNode | likely | Initializes an IString field on the object passed in ECX through 009c4550, stores the supplied dword at object offset 0x1c, and returns this. |
| `009c49a0` | TimerDataStringsScalarDeletingDtor | likely | Runs the six-string aggregate cleanup, frees the allocation only when the deleting flag is set, and otherwise returns the original pointer. |
| `009c4e60` | InsertHashListNode | likely | Allocates and initializes a node, links it at the list tail and into the bucket selected by the node hash, and increments the map count. |
| `009c4f00` | InsertHashListNode | likely | Allocates and initializes a node, links it at the list tail and into the bucket selected by its key, and increments the map count. |
| `009c5050` | InsertHashListNode | likely | Allocates and initializes an active-timer node, links it at the list tail and into the hash bucket, then increments the map count. |
| `009c5550` | CreateTimerEventAtElapsedTime | likely | Reads the current time, subtracts it from the supplied timestamp, and passes the resulting delta to the timer-event creation routine. |
| `009c5590` | StartClientTimer | likely | Finds a timer definition by name, checks for existing active entries, invokes a gated object callback when needed, inserts active state, and emits its Lua even… |
| `009c5720` | StopClientTimer | likely | Looks up active timer state by name, updates related callback state, removes the active entry, and may add a saved timer entry with current client identifiers. |
| `009c5f60` | RestoreSavedTimersForClient | likely | Walks saved records, looks up corresponding timer definitions, removes unmatched entries, and creates timer events for matching records according to the saved … |
| `009c6000` | StartAutoTimers | likely | Traverses timer definitions and starts those marked auto-start when no saved timer with the same key is found. |
| `009c63c0` | ClientTimersInitialize | likely | Sets the timer manager state to 1, loads timer definitions, and then loads saved client timers. |
| `009c63e0` | ClientTimersStart | likely | When state equals 1, changes it to 2, restores saved timers, and starts configured automatic timers; otherwise it does nothing. |
| `009c65e0` | ClientTimersManagerInit | likely | Sets manager state to zero, stores the owner pointer, and constructs definition, saved-timer, and active-timer containers. |
| `009c6ac0` | ClientTimersManagerDtor | likely | Calls 009c6670 with the manager in ECX; drains map heads at +0x10 (TimerDefinition), +0xa4 and +0x138 (two SavedTimerData collections), and +0x1cc (ActiveTimer… |
| `00bdfc00` | LocalClientActivityDefinition_DtorHelper_2 | likely | SEH-framed wrapper that calls FUN_01082860, the ClientActivityDefinition destructor body; identical to 00bdf490 apart from its exception handler label. |
| `00be58f0` | LoadActivityCategories_txt | likely | Loads the ActivityCategories.txt file: resets the IString path with the literal, reads the file through FUN_00fbf4a0 and loops over its lines (FUN_006fc270 rea… |
| `00c4a390` | ActivityLaunchRequest_FirstMember | likely | Seven-byte getter (bytes 8b 81 98 00 00 00 c3 = mov eax,[ecx+0x98]; ret): returns the head pointer of the member list embedded in the launch request at +0x94. |
| `00c4a5e0` | ActivityMember_NextNode | likely | Ten-byte getter (8b 44 24 04 8b 40 50 c2 04 00): returns the +0x50 next link of the member node passed as the argument. |
| `00c4bb70` | ActivityMemberList_UnlinkAndFree | likely | Removes one member node from the intrusive list: relinks the previous node (+0x54) and next node (+0x50) around it, updating the list head (list+4) or tail (li… |
| `00c4bc00` | ActivityMemberList_InsertTailFromDescriptor | likely | Appends a member to the intrusive list by building a new node: allocates through the list's virtual slot 2 (+8), constructs the member from the descriptor argu… |
| `00c4d9f0` | InviteResponse_SerializeAndSend | likely | Serialises an InviteResponse packet into a stack buffer and sends it: initialises the buffer builder 008d4a40 with capacity 0x10000000, writes the response fie… |
| `00c4dac0` | OwnerLaunchRequest_SerializeAndSend | likely | Serialises an OwnerLaunchRequest packet and sends it: initialises the buffer builder 008d4a40, writes the request fields with 00c4cd20 into the stack buffer, s… |
| `00c4db90` | OwnerMatchmakingRequest_SerializeAndSend | likely | Serialises an OwnerMatchmakingRequest packet and sends it, using the same buffer builder 008d4a40, field writer 00c4cd20, sender 008a9140 and release 00894560 … |
| `00c4dc60` | ReadyNotification_SerializeAndSend | likely | Serialises a MemberReady packet and sends it, using the buffer builder 008d4a40, field writer 00c4cd20, sender 008a9140 and release 00894560 sequence. |
| `00c4dd30` | QuitNotification_SerializeAndSend | likely | Serialises a MemberQuit packet and sends it, using the buffer builder 008d4a40, field writer 00c4cd20, sender 008a9140 and release 00894560 sequence. |
| `00c4e7f0` | SetFlagAndDispatchSlot4 | likely | Sets byte this+0x36c to 1, then tail-jumps through this object's vtable slot 4 with ECX unchanged. |
| `01082a90` | FormatFeaturedActivityDetails | likely | Thiscall with one stack arg (output string pointer). Passes the int64 epochs at this+0x0 and this+0x8 to 007026b0 (calendar conversion) and formats two YYYY-MM… |
| `009c0ee0` | DestroyClientActivityLaunchRequest | speculative | Calls 00c4d370 with the original this pointer still in ECX, then restores the exception state. |
| `009c46f0` | TimerEventFieldsInit | speculative | Copies several global defaults into the object and initializes an embedded IString plus a trailing flag byte. |
| `009c4880` | InitializeTimerEventNode | speculative | Initializes fields on the object passed in ECX through 009c46f0, stores the supplied dword at object offset 0x38, and returns this. |
| `00bdffe0` | InitRecordDefaultsAndStoreDword | speculative | Calls FUN_010824f0 (the eight-dword record initialiser that writes the pair of globals DAT_01cb7cf0 and DAT_01cb7cf4 into +0 and +8 with the remaining fields z… |
| `00ce75d0` | CompareRequirementInteger | speculative | Subtracts one from selector +0x1c and compares integers using equal, not-equal, less, less-or-equal, greater, or greater-or-equal. |
| `00ce7660` | CompareRequirementInteger | speculative | Subtracts one from selector +0x1c and compares integers using equal, not-equal, less, less-or-equal, greater, or greater-or-equal. |
| `00ce76f0` | CompareRequirementInteger | speculative | Subtracts one from selector +0x1c and compares integers using equal, not-equal, less, less-or-equal, greater, or greater-or-equal. |
| `00ce7780` | CompareRequirementFloat | speculative | Subtracts one from selector +0x1c and compares the float against an integer using six equality/order operators. |
| `00ce9170` | LoadActivityCategoriesThunk | speculative | Loads this+0x428 into ECX and tail-jumps to 00cfce70. |
| `00ce9180` | LoadActivityDefinitionsThunk | speculative | Loads this+0x42c into ECX and tail-jumps to 00cfd070. |
| `00ce9190` | ActivityLoaderThunk_1 | speculative | Loads this+0x428 into ECX and tail-jumps to 00cfbe20. |
| `00ce91a0` | ActivityLoaderThunk_2 | speculative | Loads this+0x428 into ECX and tail-jumps to 00cfbe20. |
| `00ce9ad0` | ActivityMapThunk | speculative | Adds 0x214 to ECX and tail-jumps to 00ce98d0. |
| `00ce9ae0` | ScheduledActivityMapThunk | speculative | Adds 0x214 to ECX and tail-jumps to 00ce99e0. |
| `00cea5f0` | ActivityLists::RefreshFeatured | speculative | Finds the Featured list, resets it, appends each activity whose featured byte at +0x31 is set, and optionally refreshes the list. |
| `00cea650` | ActivityLists::RefreshPreferred | speculative | Finds the Preferred list, resets it, tests each activity's visibility, adds visible ids, and optionally refreshes the list. |
| `00cea6c0` | ActivityLists::RefreshMemberOnly | speculative | Finds the MemberOnly list, resets it, appends activities whose byte at +0x3c is set, and optionally refreshes the list. |
| `00cea720` | ActivityLists::Rebuild | speculative | Resets All and related activity lists, adds every activity id and qualifying Featured/Preferred/MemberOnly/custom category ids, and records ranked activities. |
| `00cea940` | ScheduledActivity::Start | speculative | Finds or creates the scheduled activity source, configures it from the supplied value, invokes its virtual method at slot 14, and returns success. |
| `00cea980` | ScheduledActivity::Update | speculative | Looks up an existing scheduled activity source without creating it, then forwards the supplied value to its update helper. |
| `00cea9b0` | ScheduledActivity::Remove | speculative | Looks up an existing scheduled activity source without creating it and forwards to its removal helper when found. |
| `00cea9d0` | ActivityList::AddId | speculative | Finds or creates an activity list and appends the supplied activity id when the list is available. |
| `00ceaa20` | Activity::Reset | speculative | Looks up an activity without creating it and invokes its reset helper when found. |
| `00cfb680` | FindScheduledActivityById | speculative | Hashes the requested ID into one of 1024 buckets, walks the chain comparing node IDs, and returns a nonzero match indicator. |
| `00cfb6c0` | CompareScheduledActivityOrder | speculative | Looks up two records by ID and compares their time, position, end-time, and remaining tie-break fields to return an ordering result. |
| `00cfba20` | RemoveActivityRecordById | speculative | Searches one of 64 buckets by ID, notifies the datasource, removes the found node, and returns whether a match existed. |
| `00cfba70` | RemoveScheduledActivityById | speculative | Searches one of 512 buckets by ID, notifies the datasource, unlinks the matching node, and returns whether it existed. |
| `00cfc480` | FindClientActivityById | speculative | Searches one of 64 hash buckets for the supplied activity identifier and, when absent for a positive ID, creates a default entry and emits a datasource callbac… |
| `009c6040` | — | unknown | Finds a completed timer by key, removes active state, emits its completion event, and may add an auto-repeat timer with current client identifiers. |
| `00bdf120` | — | unknown | Calls FUN_00cea650(this+4, 0) and FUN_00cea6c0(this+4, 0), resetting both activity list data sources to the empty value; the raw bytes load this+4 and this+0x1… |
| `00bdf490` | — | unknown | SEH-framed wrapper that calls FUN_01082860, the ClientActivityDefinition destructor body (vtable reset, member cleanup and two IString releases), with a four-b… |
| `00bdf4e0` | — | unknown | SEH-framed wrapper that calls FUN_010830f0, the ClientScheduledActivityDefinition destructor body, which only rewrites the client-scheduled-activity vtable. |
| `00bdfc50` | — | unknown | SEH-framed wrapper that calls FUN_010830f0, the ClientScheduledActivityDefinition destructor body; identical to 00bdf4e0 apart from its exception handler label. |
| `00bdfdd0` | — | unknown | SEH-framed wrapper that calls FUN_01082c70 with ECX forwarded, constructing a ClientActivityDefinition on the caller-provided storage (the body installs Client… |
| `00bdfe20` | — | unknown | SEH-framed wrapper that calls FUN_01083260 with ECX forwarded, constructing a ClientScheduledActivityDefinition on the caller-provided storage (the body instal… |
| `00be5b30` | — | unknown | Initializes this, builds a local IString from the literal 'ActivityPortalSections.txt' via 00402ff0, gates on 00fbf4a0 returning nonzero, then walks the buffer… |
| `00be5d70` | — | unknown | Calls 00769710 and reads a 32-bit value at (result+4)+0x74; calls 00702670 to produce a 64-bit value; appends a node through 00be4ec0 (on *(param_1+4)); calls … |
| `00c4bf90` | — | unknown | Appends a member node to the intrusive list like 00c4bc00 but constructs the member with no descriptor (00c4ba60), then links the node after the tail and incre… |
| `00c4cc70` | — | unknown | Twenty-seven byte helper that repeatedly removes the head node of the list at param_1 through 00c4bb70 until the head pointer at +4 becomes null; it is the cle… |
| `00c4de00` | — | unknown | Serialises an InviteMemberRequest packet and sends it: same builder/writer/send/release sequence as the other launch senders, with two additional helper calls … |
| `00c4def0` | — | unknown | Serialises a KickMemberRequest packet and sends it: same builder/writer/send/release sequence, with the same two extra helper calls (008fb810, 00911c30) that w… |
| `00cea090` | — | unknown | Initializes activity hash maps, clears two pointers, allocates/constructs shared activity objects of 0x47c and 0x368 bytes, then assigns them to fields. |
| `00cea190` | — | unknown | Releases two shared objects when the global gate is enabled, clears both fields, then destroys scheduled and activity maps. |
| `00ceaa00` | — | unknown | Looks up an activity without creating it and forwards the provided value to the activity update helper when found. |
| `00ceaa40` | — | unknown | Clears scheduled activity list, walks schedule entries, applies each entry value, then invokes list refresh. |

### `game.minigame`

**ClientMiniGameManager** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009bc410` | ClientMiniGameManager::ClientMiniGameManager | confirmed | Constructor: chains GuiEventHandler and DelayedAssetLoader::StatusHandler vtables into ClientMiniGameManager::vftable, stores the owner client at +8, initialis… |
| `009bf5a0` | UpdateMiniGameManagerState | confirmed | Sets manager state to 3, runs state cleanup, and requests mini-game start when manager flag bit 3 is set. |
| `009bca60` | ClientMiniGameManager::ScalarDeletingDestructor | likely | Scalar deleting destructor for the client mini-game manager: calls the manager destructor 009bbba0, which resets the ClientMiniGameManager vtables and tears do… |
| `009bbba0` | — | unknown | Destructor: restores both ClientMiniGameManager vtables, releases the raw table pointer at +0x2bea4 (FUN_00c43440 then free), tears down the config pieces (FUN… |

**ClientTradingCardGameManager** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ad3670` | ClientTradingCardGameManager::InitializeAssets | confirmed | If offset +0x40 is zero, sets state +0x2ba90 to 1, stores a 2-float result from 00702670 at +0x2bbc0/+0x2bbc4, and lazily creates and starts the AssetsTcg reso… |
| `00ad3740` | ClientTradingCardGameManager::OnDisconnect | confirmed | Logs session ticket, launch ticket and station name to ClientTCG.log, then calls 010d00d0 if offset +0x14 is nonzero. |
| `00ad3e40` | ClientTradingCardGameManager::Shutdown | confirmed | If manager offset +0x14 is set, disables active state, removes TcgMainWindow, releases an allocated member, clears asset/resource objects and map nodes, and se… |
| `00ad4f50` | ClientTradingCardGameManager::Update | confirmed | Advances staged manager states 1 through 4, starts TradingCardAssets.txt loading, checks an asset-count threshold, updates child objects in a tree and complete… |
| `00ad6410` | ClientTradingCardGameManager::Dtor | confirmed | Restores manager vtables, shuts down TCG resources, releases embedded strings and allocator state, destroys its typed map, and is called from EH cleanup paths. |
| `00ad6510` | ClientTradingCardGameManager::ScalarDeletingDtor | confirmed | Calls manager destruction and frees the object allocation when the deleting-destructor flag bit is set. |
| `00ad47f0` | — | unknown | Creates startup ticket/session objects, initializes resource and sound state, logs realm/session/launch/station values, registers the manager callbacks, and se… |
| `00ad6300` | — | unknown | Installs manager and delayed-asset status-handler vtables, initializes map and embedded resources, sets fields and initial TCG state to zero. |

**(no class)** (55)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009b6950` | OnMiniGameObjectiveFailed | confirmed | Finds the objective record, sets status at +0x30 to 4 and reformats its text. Only when record byte +0x28 is zero does it localize ObjectiveFailed; only succes… |
| `009b9500` | MiniGameState_StartActive | confirmed | If this+0x18 (the current state) is non-zero, its +0x61 byte (in progress) is set and its +0x62 byte (started) is still clear, it sets +0x62=1, stamps the 8-by… |
| `009bd650` | StartFlashMiniGameUi 🌐 | confirmed | Logs MiniGame:StartFlashMiniGame, always constructs PacketDismountRequest and sends it when client+0x280 is nonzero, then selects ShowMicro or Show and emits t… |
| `009be7d0` | RequestStartMiniGame | confirmed | If a current state exists and its start-request flag is clear, logs and sets that flag, fires GameEvents:OnPlayerStartMiniGame, then sends sub-opcode 5 using c… |
| `009beb70` | HandleMiniGameStart 🌐 | confirmed | Hides marketplace UI, optionally sends a dismount request, removes prior state, creates and initializes the incoming state, updates flags, emits Lua start even… |
| `009bef50` | CancelMiniGame | confirmed | Logs cancellation, clears action-bar and UI state, marks current state canceled, sends mini-game sub-opcode 7, resets counters, and emits the invitation-reject… |
| `009bf0f0` | LeaveMiniGameInstance | confirmed | Logs the fixed string MiniGame:LeaveInstanceMiniGame, selects the requested state when param_2 is positive or the current state otherwise, marks an eligible st… |
| `009bf190` | LeaveMiniGame | confirmed | Finds the state, marks it leaving, closes or hides handler UI, resets local state, emits end-screen operations, and sends sub-opcode 7 for the state id. |
| `009bf5e0` | TickMiniGameManager | confirmed | Runs the manager loading state machine, advances a countdown, displays the start handler, ticks activity state, and cleans up or restarts state as conditions c… |
| `00ad32e0` | SetPair | confirmed | Stores two arguments at offsets +0x2ba98 and +0x2ba9c, then calls 009aaf30. |
| `009b2f60` | MiniGameDataRecordCtor | likely | Initializes record fields and strings, installs Array<MiniGameGroupLink*> and Array<int> vtables, zeroes counters and state, and stores the supplied argument a… |
| `009b3050` | MiniGameScoreInfoCtor | likely | Initializes string state, stores five supplied dwords at offsets +0x34 through +0x44, and releases a shared string buffer when its reference count reaches zero. |
| `009b4d00` | ParseMiniGameGroupTable | likely | Parses nonempty, non-comment lines from a text source, allocates MiniGameData records, validates entries, then inserts them through FUN_009b10c0. |
| `009b5fd0` | InitGroupElementArrayAndStrings | likely | Initializes record counters and fields, sets two IString member vtables and an Array<MiniGameGroupElementInfo> vtable, and clears member values. |
| `009b6070` | DestroyGroupElementArrayAndStrings | likely | Destroys group-element array at +0x3c, releases two reference-counted strings at +0x14 and +0x24, and frees array storage. |
| `009b62a0` | LoadMiniGameDataTables | likely | Builds paths for MiniGameData.txt, MiniGameGroupData.txt, MiniGameGroupLinks.txt and MiniGameTypeData.txt; calls row parsers 009b3290, 009b4d00, 009b3710 and 0… |
| `009b6700` | OnMiniGameObjectiveComplete | likely | Finds an objective record, marks it complete, reformats status, updates per-category counters and score totals, emits ObjectiveComplete notification, and optio… |
| `009b8010` | MiniGameState_Destroy | likely | Destructor body: runs cleanup routine FUN_009b5c10, destroys three sub-objects through FUN_0103d010, calls FUN_009b22f0(*(this+0x84)), then resets the embedded… |
| `009b8190` | MiniGameState_InitWithKey | likely | Initialises a freshly allocated state record: runs FUN_00c42a30 on it (member initialisation) and copies the first dword of param_2 into the record's +0x918 fi… |
| `009b8bc0` | MiniGameGroupRecord_Copy | likely | Copy-constructs the group/state record: copies dwords 0..4 and the byte at +0x34, copy-constructs the two string members at +0x14 and +0x24 (FUN_006f9e60 twice… |
| `009b9680` | MiniGameState_CreateAndInsert | likely | Allocates a state node through virtual slot 4 of the manager interface param_1, initialises it from param_2 through FUN_009b8190 (which copies the key into +0x… |
| `009b9780` | MiniGameState_DestroyThunk | likely | One-call wrapper: invokes the mini-game state destructor 009b8010 with the object supplied in ECX (the decompiled argument is the stack-cookie value because th… |
| `009b9b30` | ObjectiveDataState_Construct | likely | In-place constructor for the record embedded in MiniGameInfoPacket and MiniGameUpdateRewardPacket: zeroes the five-dword id block (offsets +0..+0x10, with the … |
| `009baba0` | MiniGameStateList_RemoveNode | likely | Removes one state node: unlinks it from the manager's state hash bucket chain through FUN_009abab0 (key at node+0x918 % 10, bucket link at node+0x91c), unlinks… |
| `009bace0` | MiniGameStateList_Clear | likely | Repeatedly removes the head node of the manager's state list by calling MiniGameStateList_RemoveNode (009baba0) until the head pointer at this+8 becomes null. |
| `009bda80` | SendLootWheelRotationStopped | likely | Constructs a loot-wheel-rotation-stopped mini-game packet and sends it when the client transport field at +0x280 is available. |
| `009bdaf0` | RequestTcgChallenge | likely | Checks two guards, constructs a TCG challenge request, and sends it when the client transport is available. |
| `009bdbd0` | RequestWheelInteraction | likely | Checks the current scene's wheel field and the local wheel activity count, then constructs and sends an interaction-start packet when both permit it. |
| `009bdc60` | SendMiniGamePacket | likely | Resolves state id -1 from the current state, rejects other negative state ids, constructs a family-39 mini-game packet from the supplied sub-opcode and ids, th… |
| `009be8e0` | SendMiniGameLeaveSubop6 | likely | Pushes sentinel ids and sub-opcode 6, then calls the shared mini-game packet sender. |
| `009be8f0` | SendMiniGameLeaveSubop7 | likely | Forwards the supplied id and sentinel game id with sub-opcode 7 to the shared mini-game sender. |
| `009be910` | SetMiniGameReady | likely | When the manager is active, either handles game type 0x17 locally or sends sub-opcode 8 with the current state id, then sets client ready byte +0x790 and retur… |
| `009be970` | SetMiniGameNotReady | likely | When the manager is active, either handles type 0x17 locally or sends sub-opcode 9 with the current state id, then clears client ready byte +0x790 and returns … |
| `009be9d0` | ToggleMiniGameReady | likely | When manager count is positive, sends sub-opcode 0x3b with the current state id, toggles byte +0x790, and returns 1; otherwise returns 0. |
| `009bea10` | SendMiniGameSubop25 | likely | Passes the current state id, or zero if no state exists, and the supplied second id to the shared sender with sub-opcode 0x19. |
| `009bea40` | SendMiniGameSubop26 | likely | Passes the current state id, or zero if no state exists, and the supplied second id to the shared sender with sub-opcode 0x1a. |
| `009bf4d0` | ApplyMiniGameLeaveState | likely | Copies a status byte and two values into current state, clears its three payload buffers, then calls the leave handler for its id and clears client flag +0x945. |
| `009bf590` | SendMiniGameSubop37 | likely | Pushes sentinel ids and sub-opcode 0x25, then calls the shared mini-game sender. |
| `00ad1270` | SetFlag5ef62 | likely | If byte +0x5ef62 is clear, sets it, calls 00f9b070 with the global value at 017ed1a0, and returns 1; otherwise returns 0. |
| `00ad12a0` | ClearFlag5ef62 | likely | If byte +0x5ef62 is set, clears it, calls 00f9b080, and returns 1; otherwise returns 0. |
| `00ad12d0` | SetFlag5ef61IfHandlerPresent | likely | Checks this+0x5ef48; if present, sets byte +0x5ef61 and calls 00f9c220. |
| `009ba4c0` | RewardEntry110_Copy | speculative | Copy-constructs one 0x110-byte reward-row element: copies the first dword, copy-constructs three SoeUtil::IString<unsigned short> members at +4, +0x14 and +0x3… |
| `00ad33f0` | GetEnabledState | speculative | If the object referenced at +0x10 has a nonzero dword at +0x888, returns 00b08070(param_3,1); otherwise returns that object pointer with its low byte cleared. |
| `00ad3440` | GetMusicLevels | speculative | When manager offset +0x14 and global music setting +0x6d8 are enabled, reads two values through 007bf640 using indices 0xb and 0xc, stores them at +8/+0xc and … |
| `00ad4340` | GetCardCount | speculative | Returns -1 when the argument is zero; otherwise obtains a value through 00738e70 and passes it with DAT_01762a34 to 00b07c30. |
| `00ad4390` | SelectCard | speculative | For nonzero input, releases any existing object at +0x2bbb4, resolves the input via 00738e70 and conditionally creates/stores a new object through 00b07c30; re… |
| `00939100` | — 🌐 | unknown | Queries the client subobject at this+0x2f8. If that mode query is true, calls 009ad0b0(0,0); a null result or returned object's byte +0x61==0 calls 009be8e0 an… |
| `009b3290` | — | unknown | Parses an input string into nonempty lines, skips lines beginning with #, allocates records for accepted lines, then inserts each record through FUN_009b1070. |
| `009b34f0` | — | unknown | Parses nonempty, non-comment lines from an input string, allocates a record for each accepted line, validates its fields, and inserts it through FUN_009b12f0. |
| `009b3710` | — | unknown | Parses a delimited input string into nonempty non-comment lines, constructs one record per line, and inserts it with FUN_009b11b0; conditionally processes a se… |
| `009b4ca0` | — | unknown | Zeroes dwords at byte offsets 0x00 through 0x64 and at 0x6c, leaves +0x68 untouched, then passes the pointer stored at record +0x70 to FUN_009b1660. |
| `009b4f60` | — | unknown | Walks nodes from manager+0x2c0d4 via next at node+0xc. For each node, hashes its dword at +0x4c modulo 100 and searches the first bucket table at manager+0x2c5… |
| `009ba330` | — | unknown | Scalar deleting destructor for the mini-game state record: runs the state destructor 009b8010 and then frees the object through _free when the low bit of param… |
| `009ba390` | — | unknown | Copy-constructs one 0xe8-byte reward-entry element: copies dwords 0..3, the byte at +0x10, copy-constructs the embedded RewardBundleBase at +0x18 (FUN_009b9a70… |
| `00ad4510` | — | unknown | Resolves the input, searches the manager's key structure and reference table, returns zero if absent; on a new key, increments a referenced object and inserts … |

### `game.repeating_activity`

**ClientRepeatingActivity** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00adabc0` | ClientRepeatingActivity::ClientRepeatingActivity | confirmed | Constructor overload taking an activity id: stores the id at +8, installs RepeatingActivityPlayer::vftable then ClientRepeatingActivity::vftable, initialises c… |
| `00adac80` | ClientRepeatingActivity::~ClientRepeatingActivity | confirmed | Restores ClientRepeatingActivity::vftable, releases the two reference-counted IString members at +0x38 and +0x28, then resets the vtable to RepeatingActivityPl… |
| `00adad60` | ClientRepeatingActivity::ScalarDeletingDtor | confirmed | Virtual deleting destructor: calls the non-deleting destructor 00adac80 and then _free(this) when bit 0 of the flag argument is set; it is vtable slot 0 of Cli… |
| `00adad80` | ClientRepeatingActivity::ClientRepeatingActivity | confirmed | Copy constructor copies source dwords at +8,+0xc,+0x10,+0x18,+0x1c,+0x20,+0x24 and +0x48 through +0x58; it skips source +0x14. It installs the vtables, initial… |
| `008c06d0` | ClientRepeatingActivity::ScalarDeletingDestructor | likely | Calls 0x00adac80 with this in ECX and no stack arguments, then conditionally frees this when flag bit 0 is set. |
| `00adab00` | ClientRepeatingActivity::ClientRepeatingActivity | likely | Default constructor of an activity entry: installs RepeatingActivityPlayer::vftable then ClientRepeatingActivity::vftable, sets id at +8 to -1, count at +0xc t… |
| `00adb530` | ClientRepeatingActivity_CopyCtorWrapper | likely | SEH-framed wrapper that forwards to the copy constructor 00adad80 with the source entry pointer, returning this. |

**GroupHandler** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ada6f0` | GroupHandler::SetWheel | likely | Stores the first two arguments as the entry's count and consecutive values (+0xc/+0x10), then adds the current 64-bit time from FUN_00702670 to arguments 4 and… |

**RepeatingActivityPlayer** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ad95d0` | RepeatingActivityPlayer::~RepeatingActivityPlayer | confirmed | Resets the object's vtable pointer to RepeatingActivityPlayer::vftable and returns; the 7-byte body (c7 01 cc a5 81 01 c3) has no member cleanup and it is reac… |
| `00ad9de0` | RepeatingActivityPlayer::ScalarDeletingDtor | confirmed | Virtual deleting destructor: stores RepeatingActivityPlayer::vftable back into the object and calls _free(this) when the flag argument has bit 0 set; it is vta… |

**(no class)** (16)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00adb400` | ActivityList_LogAll | confirmed | Builds a log string: when the list count at +0x10 is below 1 it assigns No activities available., otherwise it formats %d activities: and appends one %s id=%d … |
| `00ada790` | ActivityList_FindByName | likely | Walks the singly linked list at this+8 following the next pointer at element+0x60 and returns the first element whose string field at +0x3c compares equal (FUN… |
| `00ada800` | ActivityList_ApplyStateMessage | likely | Walks the list at this+8 (next at +0x60) looking for the element whose id at +8 equals the state message field at +0xc, then forwards the message fields +0x10.… |
| `00ada830` | ActivityList_CountForName | likely | Looks the entry up by name with 00ada790; if found returns its count field (+0xc), otherwise sums the +0xc fields of every list element whose secondary name fi… |
| `00ada880` | ActivityList_FindByValue58 | likely | Walks the list at this+8 (next at +0x60) and returns the count field (+0xc) of the first element whose dword at +0x58 equals the argument, or 0 when nothing ma… |
| `00adb900` | ActivityList_AppendCopy | likely | Calls the list virtual entry at vtable offset +8 with ECX=this (the apparent stack-cookie argument in pseudocode is SEH bookkeeping), passes the source entry t… |
| `00adb990` | RepeatingActivity_OnAddMessage | likely | Searches the list at this+8 for an entry with id equal to the message field +0xc; when found it forwards message fields +0x10..+0x1c to GroupHandler::SetWheel,… |
| `00adc9b0` | RepeatingActivity_RequestClearAll 🌐 | likely | Asks the client's context interface (vtable slot 0xb4) for the current clear-context value, builds a clear request with id -1 from it and sends it through 00ad… |
| `00adca40` | ActivityList_ClearByName 🌐 | likely | Finds an entry by name, builds a type-4 clear object with its id and a context value, sends only when connected, then zeroes +0xc/+0x10 and restores defaults a… |
| `00adcb30` | RepeatingActivity_RequestSetConsecutive 🌐 | speculative | Builds a type-5 set-consecutive request with 00ad99e0 from the two arguments and sends it through the tunnel wrapper 00adc4d0 when the connection flag at clien… |
| `00adcbb0` | RepeatingActivity_RequestAddCount 🌐 | speculative | Builds a type-6 addcount request with 00ada200 from the two arguments and sends it through the tunnel wrapper 00adc5e0 when the connection flag is set. |
| `00adcc40` | RepeatingActivity_RequestSetCount 🌐 | speculative | Builds a type-8 setcount request with 00ada3d0 from the two arguments and sends it through the tunnel wrapper 00adc6f0 when the connection flag is set. |
| `00adccd0` | RepeatingActivity_RequestCountRemaining 🌐 | speculative | Builds a type-9 count-remaining query with 00ada4b0 from the argument and sends it through the tunnel wrapper 00adc800 when the connection flag is set. |
| `00ada8b0` | — | unknown | Looks the entry up by name (00ada790) and returns its +0x50 deadline minus the current time from FUN_00702670; when no entry matches, scans all elements with +… |
| `00adb250` | — | unknown | Chooses the SUCCESSFUL or FAILED format string from the packet's success byte at +0x20, formats the 64-bit Guid (+0xc/+0x10) and the count fields (+0x18/+0x1c)… |
| `00adb330` | — | unknown | Formats the packet's 64-bit Guid (+0xc/+0x10) and the count/activity-id dwords (+0x18/+0x1c) through the Target(Guid=%lld) has %d(count) remaining... format st… |

### `minigame`

**ClientMiniGameManager** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009b0c70` | ClientMiniGameManager::UpdateStateFlags | confirmed | Compares two stored state IDs with param_2 and sets or clears byte flags 0x40 and 0x80 based on param_3+4. |

**ControllerFlashMiniGame** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00af68d0` | ControllerFlashMiniGame::Ctor | confirmed | Calls Controller::Controller (0094f6c0) with the forwarded arguments, stores the ControllerFlashMiniGame vtable, calls 00952030 with count 4 (RegisterSlotBindi… |
| `00af69b0` | ControllerFlashMiniGame::DtorBody | confirmed | Stores the ControllerFlashMiniGame vtable, calls virtual slot +0x24 on the object at +0x74 +0xc0 and calls 00705200 with 1 when it returns nonzero, then calls … |
| `00af6a50` | ControllerFlashMiniGame::ScalarDtor | confirmed | Virtual slot 7, scalar deleting destructor shape: calls the destructor body 00af69b0, then calls _free when bit 0 of the argument is set. |
| `00af6a70` | ControllerFlashMiniGame::ProcessActions | likely | Virtual slot 50. Walks the action list at +0xc8; for each button entry (kind 0 with flag +0xd) whose id is in 0x13..0x17 it calls DispatchActionBarSlot (008a7e… |

**(no class)** (35)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a5850` | IsMiniGameFlashType | confirmed | Predicate over two query calls: calls FUN_009ad0b0(0,0); if that returns 0 the result is 0, otherwise it queries FUN_009ad290(0) and returns 1 only for the val… |
| `008a6b50` | StartPendingMiniGameIfState2 | confirmed | When the dword at this+0x700 equals 2, tail-jumps to 009be7d0 with this in ECX; otherwise returns without action. |
| `009ad480` | ApplyMiniGameGameOverResult | confirmed | Logs game-over, resolves current or keyed state, stores won byte and three result dwords, then invokes cleanup/update calls. |
| `009ad830` | SetEndGameDialogClosed | confirmed | Logs MiniGame:SetEndGameDialogClosed; clears current-state byte +0x2c0 and sets byte +0x2c1. |
| `009b0cc0` | SetMiniGameReadyAndDispatchEvent | confirmed | Logs MiniGame:SetMiniGameReady, resolves HandlerMiniGameStart, appends :setReady and dispatches it when available, then sets bit 0x04 in byte this+0x0c. |
| `00c42440` | ObjectiveStatus_FormatText | confirmed | Switches on the status dword at +0x30 and rewrites the display IString at +0x34 through the string-table object returned by FUN_00769710: 1 = ObjectiveStatusNo… |
| `009acf60` | PortalRows_GetCount | likely | Returns the 32-bit value at this+0x2c5b4. |
| `009acf70` | PortalRow_GetNext | likely | Returns the 32-bit pointer-sized field at param_1+0x0c. |
| `009ad0b0` | MiniGameStateLookup | likely | Returns current state for IDs below one; otherwise searches ten buckets by state ID at +0x918. |
| `009ad100` | MiniGameStateLookupRepeat | likely | Returns current state for IDs below one; otherwise searches ten buckets and compares each node ID at +0x918. |
| `009ad150` | GetCurrentStatePointer | likely | Returns the pointer stored at param_1+0x18. |
| `009ad160` | GetCurrentStateScoreBlock | likely | Returns current-state pointer+0x2cc when present; otherwise returns null. |
| `009ad3c0` | HasPositiveCountAt20 | likely | Returns whether the signed integer at param_1+0x20 is greater than zero. |
| `009ad420` | SetCurrentStateFlag2b8 | likely | When current state exists, stores byte one at state+0x2b8. |
| `009ad8d0` | GetCurrentStateField0c | likely | Returns current-state field +0x0c if present; otherwise returns zero. |
| `009ad8f0` | GetCurrentStateField4 | likely | Returns current-state field +4 if present; otherwise returns zero. |
| `009ad900` | GetCurrentStateField0 | likely | Returns first dword of current state if present; otherwise returns zero. |
| `009aef40` | CountGroupEntriesNamedChallenge | likely | Finds a keyed group and counts its elements whose element type matches param_3 and string field equals challenge. |
| `009af660` | InitMiniGameDataRecord | likely | Initializes a record's links, counters, flags, and several IString<char> members to empty/default values. |
| `009af7b0` | InitMiniGameRecordVariant | likely | Initializes links, counters, supplied field param_2, and two empty IString<char> members. |
| `009af860` | InitMiniGameRecordVariant2 | likely | Initializes links, counters, supplied field, an empty string, and trailing byte fields. |
| `009af900` | InitMiniGameGroupElementRecord | likely | Initializes record ID, three empty strings, numeric fields, and default boolean and counter values. |
| `009afd00` | MiniGameDataRecord_Dtor | likely | Restores five IString<char> members and decrements/releases each owned buffer when its reference count reaches zero. |
| `009afe40` | MiniGameGroupElementInfo_StringDtor | likely | Restores an IString<char> member and releases its owned buffer when the reference count reaches zero. |
| `009afeb0` | MiniGameRecordTwoStringDtor | likely | Restores two IString<char> members and releases their owned buffers when references reach zero. |
| `009b0b20` | MiniGameDataRecord_Dtor | likely | Clears two integer hash maps, frees integer and group-link arrays, then releases two IString members. |
| `009b0db0` | DispatchMiniGameExitEvent | likely | When current state exists, resolves HandlerMiniGameStart, appends :exit, and dispatches the resulting UI event. |
| `009b1440` | MiniGameGroupElementInfo_CopyCtor | likely | Copies scalar fields and bytes and deep-copies three IString members from the source record. |
| `009b1560` | MiniGameScoreInfo_CopyCtor | likely | Initializes a record through FUN_008dffc0, then copies five dwords at offsets +0x34 through +0x44. |
| `009b16e0` | MiniGameStateListInsertTail | likely | Links state at the tail of manager list, prepends it to ID modulo-ten bucket, increments count, and returns state. |
| `00c42400` | FindRecordByKey_Stride0x110 | likely | Linear scan of the goal-record array embedded at this+0x80 with the count at this+0x84: compares each record's first dword to the key argument, advancing 0x44 … |
| `00c42740` | — | unknown | Applies a source record to the goal state in ECX: copies the source dwords +0xd4 and +0xd8 into the state's count (+0x44) and total (+0x48), hands the source's… |
| `00c42870` | — | unknown | Builds a 0x110-byte goal record from a data record: copies the id dword and the +0xd4/+0xd8 counters, installs three IString<unsigned_short> members (vtable at… |
| `00c42a30` | — | unknown | Constructor of the minigame object: writes the class vtable-free scalar header (0x14 at +0, zeroes through +0x20), installs IString<char> members at +0x14/+0x2… |
| `00c42c60` | — | unknown | Rebuilds a minigame's goal list from a 0x270-byte source record: copies scalars (+0x94/+0x98/+0x9a fields and the +0x265..+0x267 bytes), inits two subobjects v… |

### `minigame.activity`

**ClientActivityLaunchGaqRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bb37b0` | ClientActivityLaunchGaqRequest::VSlot0Decline | likely | Vtable slot 0 delegate (decoded: mov eax,[ecx+8]; mov ecx,[ecx+4]; mov ecx,[ecx+0x218]; push eax; call 009c1c60; ret): loads the manager at (*(this+4))+0x218 i… |
| `00bb37d0` | ClientActivityLaunchGaqRequest::VSlot1Accept | likely | Vtable slot 1 delegate (same shape as slot 0): loads the manager at (*(this+4))+0x218 into ECX and calls 009c19a0 with the request id at this+8. |
| `00bb37f0` | ClientActivityLaunchGaqRequest::VSlot2Decline2 | likely | Vtable slot 2 delegate: identical to slot 0, it loads the manager at (*(this+4))+0x218 into ECX and calls 009c1c60 (decline) with the id at this+8. |

**ClientMiniGameActivityLaunchProcessor** (17)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bb36d0` | ClientMiniGameActivityLaunchProcessor::GetName | confirmed | Six-byte vtable slot 1 accessor (mov eax,0x018251c4; ret): returns the constant C string 'Minigame', the name this processor registers under. |
| `00bb36e0` | ClientMiniGameActivityLaunchProcessor::~ClientMiniGameActiv… | confirmed | Destructor body: installs ClientMiniGameActivityLaunchProcessor::vftable and then immediately IClientActivityLaunchProcessor::vftable into this+0 (the derived-… |
| `00bb3810` | ClientMiniGameActivityLaunchProcessor::ScalarDeletingDtor | confirmed | Scalar deleting destructor at vtable slot 0: installs ClientMiniGameActivityLaunchProcessor::vftable, then IClientActivityLaunchProcessor::vftable, and when bi… |
| `00bb3880` | ClientMiniGameActivityLaunchProcessor::CancelGaq | confirmed | Vtable slot 4: when the pending request at this+8 exists, logs 'Canceling GAQ.' to ClientActivityLaunchRequests.log (00d0a5a0), cancels it with 00a0dc00 and cl… |
| `00bb3e30` | ClientMiniGameActivityLaunchProcessor::OnInviteAutoDeclined | confirmed | Vtable slot 6 event hook: assembles a four-argument Lua event record (actor id from the passed record +0xa4, the two string/pair arguments and the extra dword … |
| `00bb4030` | ClientMiniGameActivityLaunchProcessor::OnInviteSent | confirmed | Vtable slot 8 builds five event values: actor id from +0xa4 (or 0), a tagged value from param_2/param_3, param_4, a value tagged 5 whose byte payload is param_… |
| `00bb4290` | ClientMiniGameActivityLaunchProcessor::OnMemberRemoved | confirmed | Vtable slot 12 event hook: builds a four-argument Lua event record (actor id from +0xa4 plus the three caller arguments) and fires 'ActivityEvents:OnMemberRemo… |
| `00bb4490` | ClientMiniGameActivityLaunchProcessor::OnInviteResult | confirmed | Vtable slot 9 event hook: assembles a five-argument Lua event record (actor id from +0xa4, the caller's string and dword arguments, a boolean marker stored as … |
| `00bb46f0` | ClientMiniGameActivityLaunchProcessor::OnKicked | confirmed | Vtable slot 10 event hook: builds a three-argument Lua event record (actor id from the passed record at +0xa4 plus the caller's string and dword arguments) and… |
| `00bb4880` | ClientMiniGameActivityLaunchProcessor::OnMemberKicked | confirmed | Vtable slot 11 event hook: builds a five-argument Lua event record (actor id from +0xa4, the caller's string and dword arguments and a second string/dword pair… |
| `00bb4a80` | ClientMiniGameActivityLaunchProcessor::OnMatchmakingStart | confirmed | Vtable slot 13 event hook: builds a one-argument Lua event record holding the actor id from the passed record (+0xa4) and fires 'ActivityEvents:OnMatchmakingSt… |
| `00bb4b90` | ClientMiniGameActivityLaunchProcessor::OnMatchmakingLaunch | confirmed | Vtable slot 14 event hook: builds a one-argument Lua event record with the actor id from the passed record (+0xa4) and fires 'ActivityEvents:OnMatchmakingLaunc… |
| `00bb4ca0` | ClientMiniGameActivityLaunchProcessor::OnMatchmakingCancel | confirmed | Vtable slot 15 event hook: builds a one-argument Lua event record with the actor id from the passed record (+0xa4) and fires 'ActivityEvents:OnMatchmakingCance… |
| `00bb3660` | ClientMiniGameActivityLaunchProcessor::ClientMiniGameActivi… | likely | Constructor: installs IClientActivityLaunchProcessor::vftable, zeroes +8 (the pending GAQ request pointer), installs ClientMiniGameActivityLaunchProcessor::vft… |
| `00bb4f50` | ClientMiniGameActivityLaunchProcessor::JoinMultiplayerMiniG… | likely | Vtable slot 7: prepares a reader (009b9b30/009b5fd0), reads the pending launch record with 00bb4db0 and, when it validates, performs the join through 009aae60 … |
| `00bb3bf0` | — | unknown | Vtable slot 5 event hook: builds a one-LuaParam argument array holding the actor id from the passed record (+0xa4), fires 'ActivityEvents:OnLaunchCanceled' wit… |
| `00bb4e40` | — | unknown | Vtable slot 3: prepares a reader with 009b9b30/009b5fd0, validates the pending record with 00bb4db0 and then either performs the direct launch path (009aae60 w… |

**IClientActivityLaunchProcessor** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bb3620` | IClientActivityLaunchProcessor::~IClientActivityLaunchProce… | confirmed | Seven-byte destructor body (mov dword [ecx],vtable; ret): installs IClientActivityLaunchProcessor::vftable and returns; it is called only from the three unwind… |
| `00bb3630` | IClientActivityLaunchProcessor::ScalarDeletingDtor | confirmed | Scalar deleting destructor at vtable slot 0: rewrites IClientActivityLaunchProcessor::vftable into this+0 and, when bit 0 of the flag argument is set, releases… |

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bb38c0` | BuildActivityLaunchInviteDialog | confirmed | Builds the invite dialog for an incoming activity launch: fetches the localised title through vtable slot 0x14 of the localisation object at [this+4]+4 with th… |
| `00bb4db0` | ReadFieldsAndTailRange | likely | Consumes one record from the packet reader at param_1: takes the pending byte range [reader+0x4c, reader+0x4c+reader+0x50], reads one value with 009bdd70, then… |
| `00bb3740` | — | unknown | When the pending request pointer at this+8 is non-zero: writes 'Canceling GAQ.' through the logger 00d0a5a0 using the log file 'ClientActivityLaunchRequests.lo… |

### `minigame.derby`

**VehicleDemolitionDerbyProcessor** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009f39b0` | — 🌐 | unknown | Initializes a temporary derby base-packet record, bounds-checks and reads two u16 header fields from the supplied stream through 009ee770, then switches on the… |

### `minigame.draft`

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012d3c50` | — | unknown | Fastcall on this: when this+8 (the NetworkService) is set it runs 012f2470 with ECX = this+8; then, when any of the timers at +0x20, +0x34 or +0x40 is positive… |
| `012d4730` | — | unknown | Thiscall with one int argument: stores it at this+0x40; when it is positive it starts a 64-bit timestamp into +0x44/+0x48 (013c5d50); allocates a 0x14 event, s… |

### `net.minigame`

**BaseMiniGameAdminPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a97c0` | BaseMiniGameAdminPacket::BaseMiniGameAdminPacket 🌐 | confirmed | Constructor: installs BasePacket::vftable, writes this[1] = 0x5f (base opcode 95), copies param_2 into this[2], then installs BaseMiniGameAdminPacket::vftable … |
| `009ac450` | DeletingDtor_009ac450 | likely | Scalar deleting destructor for a BasePacket-derived minigame packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object … |

**BaseShipCombatPacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ca68f0` | BaseShipCombatPacket::ParseApplyRecord_00ca68f0 | likely | Constructs a local packet via 00ca3870 and range-tests (param_1, param_2) through 00ca5cb0. On the valid path with a non-sentinel id it looks up the object wit… |
| `00ca64e0` | — | unknown | Constructs a local packet via 00ca3df0, applies a (param_1, param_2) length test, calls 00ca59a0 to parse, and when the parsed id pair differs from DAT_01bef6b… |
| `00ca66a0` | — | unknown | Constructs a local packet via 00ca3f10, runs a range test through 00ca5a60 on (param_1, param_2) with a non-null check on param_1, and on the valid path calls … |
| `00ca6c90` | — | unknown | Takes two float arguments; when the first is non-zero it constructs a local packet via 00ca3980, range-tests the pair through 00ca5f90, looks up the object via… |

**ClientMiniGameManager** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009b8f90` | ClientMiniGameManager::HandleGameDataUnlockedPacket | confirmed | Parses the message: shared 5-field header (FUN_009b2340), a 32-bit length-prefixed blob (FUN_008bfc00), and then three counted sections read from inside that b… |

**MiniGameCreateGameResultPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009aa230` | MiniGameCreateGameResultPacket::MiniGameCreateGameResultPac… 🌐 | confirmed | Constructor: calls the minigame base constructor FUN_008a4420 with sub-opcode 0x43 and three -1 header words, installs MiniGameCreateGameResultPacket::vftable … |
| `009ac930` | DeletingDtor_009ac930 | likely | Scalar deleting destructor for a BasePacket-derived minigame packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object … |

**MiniGameDataUnlockedPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a98c0` | MiniGameDataUnlockedPacket::MiniGameDataUnlockedPacket 🌐 | confirmed | Constructor: calls the minigame base constructor FUN_008a4420 with sub-opcode 2 and three -1 payload words, then installs MiniGameDataUnlockedPacket::vftable a… |
| `009ac540` | DeletingDtor_009ac540 | likely | Scalar deleting destructor for a BasePacket-derived minigame packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object … |

**MiniGameGameOverPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a9ab0` | MiniGameGameOverPacket::MiniGameGameOverPacket 🌐 | confirmed | Constructor: calls the minigame base constructor FUN_008a4420 with sub-opcode 0x12, writes byte param_3 at this+0x18, installs MiniGameGameOverPacket::vftable … |
| `009ac630` | DeletingDtor_009ac630 | likely | Scalar deleting destructor for a BasePacket-derived minigame packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object … |

**MiniGameKnockOutPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a99e0` | MiniGameKnockOutPacket::MiniGameKnockOutPacket 🌐 | confirmed | Constructor: calls the minigame base constructor FUN_008a4420 with sub-opcode 0x17, three payload words (param_2 plus two -1 words), then installs MiniGameKnoc… |
| `009ac5d0` | DeletingDtor_009ac5d0 | likely | Scalar deleting destructor for a BasePacket-derived minigame packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object … |

**MiniGameLootWheelOnRotationStoppedPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009aa0c0` | MiniGameLootWheelOnRotationStoppedPacket::MiniGameLootWheel… 🌐 | confirmed | Constructor: calls the minigame base constructor FUN_008a4420 with sub-opcode 0x2e and three -1 header words, then installs MiniGameLootWheelOnRotationStoppedP… |
| `009ac870` | DeletingDtor_009ac870 | likely | Scalar deleting destructor for a BasePacket-derived minigame packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object … |

**MiniGameMessagePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a9bc0` | MiniGameMessagePacket::MiniGameMessagePacket 🌐 | confirmed | Constructor: calls the minigame base constructor FUN_008a4420 with sub-opcode 0xf and two -1 payload words, installs MiniGameMessagePacket::vftable, stores par… |
| `009ac690` | DeletingDtor_009ac690 | likely | Scalar deleting destructor for a BasePacket-derived minigame packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object … |

**MiniGameStarsEarnedPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a9cb0` | MiniGameStarsEarnedPacket::MiniGameStarsEarnedPacket 🌐 | confirmed | Constructor: calls the minigame base constructor FUN_008a4420 with sub-opcode 0x15 and param_2/param_3 payload words, installs MiniGameStarsEarnedPacket::vftab… |
| `009ac6f0` | DeletingDtor_009ac6f0 | likely | Scalar deleting destructor for a BasePacket-derived minigame packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object … |

**MiniGameStoreAdvertisementWorldClient** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a9f80` | MiniGameStoreAdvertisementWorldClient::MiniGameStoreAdverti… 🌐 | confirmed | Constructor: calls the minigame base constructor FUN_008a4420 with sub-opcode 0x2c and three -1 header words, zeroes this[6], installs MiniGameStoreAdvertiseme… |
| `009ac810` | DeletingDtor_009ac810 | likely | Scalar deleting destructor for a BasePacket-derived minigame packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object … |

**MiniGameUpdateGameTimeScalar** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009aa170` | MiniGameUpdateGameTimeScalar::MiniGameUpdateGameTimeScalar 🌐 | confirmed | Constructor: calls the minigame base constructor FUN_008a4420 with sub-opcode 0x14 and three -1 header words, installs MiniGameUpdateGameTimeScalar::vftable an… |
| `009ac8d0` | DeletingDtor_009ac8d0 | likely | Scalar deleting destructor for a BasePacket-derived minigame packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object … |

**PlayClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0092f300` | PlayClient::SendClosedMinigameEndScreen | speculative | Constructs CommandPacketClosedMinigameEndScreen and, when client field +0x280 is nonzero, sends it through 00925c90. |

**(no class)** (25)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009b9550` | HandleMiniGameGameStarted | confirmed | Logs 'MiniGame:OnGameStarted' and looks the state up by StateId (param_2 < 1 uses this+0x18; otherwise it searches the hash at this+0x24+(id%10)*4 for node[0x2… |
| `009ba720` | HandlerMiniGameRound_SetEndLevelData 🌐 | confirmed | Constructs a temporary MiniGameRewardStatusPacket (family 39 sub-opcode 22 vtable plus FUN_008a4420(0x16,-1,-1,-1)) and parses the incoming buffer with FUN_009… |
| `009bb2d0` | HandlerMiniGameStatus_Populate | confirmed | When a current state exists it saves its +0xc id and +0x2b9 byte and removes it through 009baba0; looks the incoming group id (param_2+0x25c) up in the manager… |
| `009bd240` | HandleMiniGameLeave 🌐 | confirmed | Leave/close handler. If the current state exists and byte +0x61 is set, constructs EncounterParticipantRequestExitPacket through 008a31a0 with active launch in… |
| `009be6b0` | SendMiniGameFlashPayload | confirmed | When manager count at +0x20 is positive, logs MiniGame:SendPayload, constructs MiniGamePayloadPacket using the current state id or zero, copies exactly param_3… |
| `009a9840` | PacketDtor_009a9840 | likely | Destructor stub: rewrites *this to BasePacket::vftable and returns; no member destruction and no delete. Called only from MSVC unwind funclets (13 of them). |
| `009a9930` | PacketDtor_009a9930 | likely | Destructor stub: rewrites *this to BasePacket::vftable and returns; no member destruction and no delete. Called from an MSVC unwind funclet. |
| `009a9a60` | PacketDtor_009a9a60 | likely | Destructor stub: rewrites *this to BasePacket::vftable and returns; no member destruction and no delete. Called from an MSVC unwind funclet. |
| `009a9b30` | PacketDtor_009a9b30 | likely | Destructor stub: rewrites *this to BasePacket::vftable and returns; no member destruction and no delete. Called from two MSVC unwind funclets and from the pack… |
| `009a9c40` | PacketDtor_009a9c40 | likely | Destructor stub: rewrites *this to BasePacket::vftable and returns; no member destruction and no delete. Called from an MSVC unwind funclet. |
| `009a9d20` | PacketDtor_009a9d20 | likely | Destructor stub: rewrites *this to BasePacket::vftable and returns; no member destruction and no delete. Called from an MSVC unwind funclet. |
| `009aa000` | PacketDtor_009aa000 | likely | Destructor stub: rewrites *this to BasePacket::vftable and returns; no member destruction and no delete. Called from the packet builder 009bfea0 and an MSVC un… |
| `009aa050` | PacketDtorWithRewardMember_009aa050 | likely | Destructor variant: calls FUN_0103d010 (destructor of a member/base owning RewardBundleBase::vftable and SoeUtil list members) under the stack-cookie check and… |
| `009aa120` | PacketDtor_009aa120 | likely | Destructor stub: rewrites *this to BasePacket::vftable and returns; no member destruction and no delete. Called from an MSVC unwind funclet. |
| `009aa1e0` | PacketDtor_009aa1e0 | likely | Destructor stub: rewrites *this to BasePacket::vftable and returns; no member destruction and no delete. Called from the packet builder 009bfea0 and an MSVC un… |
| `009aa290` | PacketDtor_009aa290 | likely | Destructor stub: rewrites *this to BasePacket::vftable and returns; no member destruction and no delete. Called from the packet builder 009bfea0 and an MSVC un… |
| `009b87a0` | HandleMiniGameKnockOut | likely | When this+0x18 (the event context) is non-zero: constructs a temporary MiniGameKnockOutPacket (FUN_009a99e0(-1,0,0)), parses two i32 words through FUN_009b4770… |
| `009b8c50` | HandleMiniGameDataLockedPacket | likely | Constructs a temporary MiniGameDataLockedPacket (FUN_009afa50) and parses the body with FUN_009b7da0 (three id arrays). For every id in the first array it remo… |
| `009bead0` | SendMiniGameFlashCommandAdminPacket | likely | Uses the current state id or zero, constructs MiniGameFlashCommandAdminPacket from that id and the two supplied values, sends it when the state id is nonnegati… |
| `00cd5000` | WriteHandleAndHeader | speculative | Thiscall with one stack argument (stream pointer). Calls 00b72160 on the stream, calls 008fb810 with this+0x10, then appends min(stream[0x806] - *(*stream+8), … |
| `009b8630` | — | unknown | Constructs a temporary MiniGameMessagePacket (FUN_009a9bc0(-1,0,0)), parses the buffer through FUN_009b4700 (5-field header, i32 at +0x18, byte flag at +0x1c) … |
| `009b8880` | — | unknown | When the event context at +0x18 is non-zero: constructs a temporary MiniGameStarsEarnedPacket (FUN_009a9cb0(-1,0,0)), parses the body with FUN_009b5dd0 into a … |
| `009b89e0` | — | unknown | Constructs a temporary MiniGamePayloadPacket (FUN_009b0960), parses the shared 5-field record (FUN_009b2340) plus a count-prefixed byte array (FUN_009538e0) an… |
| `009bb030` | — | unknown | If there is no current state it logs 'ClientMiniGameManager: No Minigame State, possibly tutorial exit.'; otherwise it constructs a temporary end-score packet … |
| `009bca80` | — | unknown | Constructs a MiniGameGroupInfoPacket (FUN_009b8580) over the incoming buffer, reads the shared header (FUN_009b2340) and the 76-byte group record into a stack … |

### `net.minigame.soccer`

**SoccerBall** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cde9b0` | SoccerBall::ApplyOrSendImpulse 🌐 | confirmed | Calls the base soccer-ball method, selects a component handle by param_5 and invokes its virtual operation; when param_6 is nonzero, fills the 36-byte SoccerPa… |

**SoccerPlayer** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cdc4a0` | SoccerPlayer::SendUpdatePlayerPos 🌐 | confirmed | Builds SoccerPacketUpdatePlayerPos from this object's identifiers and position-related fields, then calls the payload sender when the gateway handle is present. |
| `00cdc5c0` | SoccerPlayer::SendUpdatePlayerState 🌐 | confirmed | When enabled and this player is in the sendable state, saves the previous state, fills guid/state/auxiliary/vector fields, and submits SoccerPacketUpdatePlayer… |
| `00cdc710` | SoccerPlayer::SendFreeBall 🌐 | confirmed | If the soccer context and ball pointer exist, obtains a four-dword value, fills guid and auxiliary fields, and sends SoccerPacketFreeBall when the gateway hand… |
| `00cdc810` | SoccerPlayer::SendUpdatePlayerControls 🌐 | confirmed | Optionally flips the two input floats, compares them and the integer against cached fields, sends SoccerPacketUpdatePlayerControls on change, then forwards all… |
| `00cdc9a0` | SoccerPlayer::SendSoccerBallUpdate 🌐 | confirmed | Copies a sixteen-dword transform and related ball flags into SoccerPacketUpdateSoccerBall, then submits it when the gateway handle exists. |

**SoccerProcessor** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b80ac0` | SoccerProcessor::DispatchFamily76Packet 🌐 | confirmed | Virtual slot [4] (+0x10) of SoccerProcessor::vftable. Takes a buffer range (param_2..param_3), builds a BaseSoccerPacket temporary with FUN_00b73040, reads the… |

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b83380` | — | unknown | Thiscall with two stack arguments (record pointer, flag 1), ret 8. Creates a PacketWriter on the stack (FUN_008d4a40 with 0x10000000), serializes the record at… |
| `00b83490` | — | unknown | Same tunnel template as TunnelSerializeRecordA, with a 0x54-byte record at the first argument. Serializer FUN_00b7fcf0 writes 4-byte fields from +0x0c to +0x30… |
| `00b835a0` | — | unknown | Same tunnel template as TunnelSerializeRecordA, thiscall with a record pointer and flag 1, ret 8. The record (first argument, moved to ECX) is serialized by FU… |

### `ui.activity`

**ClientActivityCategoryDatasource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cfb370` | ClientActivityCategoryDatasource::GetColumnName | confirmed | Returns Id, NameId, DescriptionId, ThumbnailImageFilename, or MysteryChestIcon for the corresponding column index. |
| `00cfd340` | ClientActivityCategoryDatasource::~ClientActivityCategoryDa… | confirmed | Restores category datasource vtables, destroys its embedded table holder, unlinks its row nodes, destroys both integer maps, and releases the holder object. |
| `00cfd630` | ClientActivityCategoryDatasource::ClientActivityCategoryDat… | confirmed | Initializes the base with the Activities.Ref.Categories source, installs category datasource vtables, then initializes two embedded integer-keyed maps. |
| `00cfd6c0` | ClientActivityCategoryDatasource::ScalarDeletingDtor | confirmed | Calls the category datasource destructor and frees the object only when deleting-flag bit 0 is set; otherwise returns the object pointer. |

**ClientActivityDatasource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cfb090` | ClientActivityDatasource::GetColumnName | confirmed | Returns field labels for activity ID, localized names, category, image, join and feature flags, membership, image filenames, difficulty, and chest icon. |
| `00cfc180` | ClientActivityDatasource::GetCell | confirmed | Resolves the activity row and underlying catalogue record, then dispatches by column to return activity flags, IDs, category, counts, filenames, and related me… |
| `00cfd1a0` | ClientActivityDatasource::~ClientActivityDatasource | confirmed | Restores the datasource vtables, destroys its embedded table holder, repeatedly unlinks remaining row nodes, destroys the integer map, and releases the holder … |
| `00cfd4f0` | ClientActivityDatasource::ClientActivityDatasource | confirmed | Initializes the shared GUI-data-source base, installs ClientActivityDatasource vtables at the object and two embedded offsets, then initializes the integer-key… |
| `00cfd570` | ClientActivityDatasource::ScalarDeletingDtor | confirmed | Calls the ClientActivityDatasource destructor and frees the object only when deleting-flag bit 0 is set; otherwise returns the object pointer. |

**ClientActivityPortalSectionDatasource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cfb3d0` | ClientActivityPortalSectionDatasource::GetColumnName | confirmed | Returns Id, NameId, or DescriptionId for indices 0, 1, and 2; other indices return an error label. |
| `00cfd420` | ClientActivityPortalSectionDatasource::~ClientActivityPorta… | confirmed | Restores portal-section datasource vtables, destroys its embedded table holder, unlinks remaining rows, destroys the integer map, and releases the holder objec… |
| `00cfd6e0` | ClientActivityPortalSectionDatasource::ClientActivityPortal… | confirmed | Initializes the base with Activities.Ref.PortalSections, installs portal-section datasource vtables, then initializes the embedded integer-keyed map. |
| `00cfd760` | ClientActivityPortalSectionDatasource::ScalarDeletingDtor | confirmed | Calls the portal-section datasource destructor and frees the object only when deleting-flag bit 0 is set; otherwise returns the object pointer. |

**ClientScheduledActivityDatasource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cfb1a0` | ClientScheduledActivityDatasource::GetColumnName | confirmed | Returns scheduled activity time fields followed by the base activity identity, label, category, image, membership, and filename fields. |
| `00cfd270` | ClientScheduledActivityDatasource::~ClientScheduledActivity… | confirmed | Restores the scheduled datasource vtables, destroys its embedded table holder, removes remaining scheduled-row nodes, destroys the map, and releases the holder… |
| `00cfd590` | ClientScheduledActivityDatasource::ClientScheduledActivityD… | confirmed | Initializes the shared GUI-data-source base, installs scheduled-activity datasource vtables at the object and two embedded offsets, then initializes its 512-bu… |
| `00cfd610` | ClientScheduledActivityDatasource::ScalarDeletingDtor | confirmed | Calls the scheduled-activity datasource destructor and frees the object only when deleting-flag bit 0 is set; otherwise returns the object pointer. |
| `00cfbda0` | ClientScheduledActivityDatasource::RefreshRows | speculative | Destroys embedded table state, reorders records with FUN_00cfb6c0, then emits each ordered record through a datasource callback. |

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cfbd10` | CleanupActivityDatasourceRows | speculative | Destroys its embedded table state, then repeatedly releases records referenced by the linked collection at offset 0x228. |
| `00cfbd70` | CleanupScheduledDatasourceRows | speculative | Destroys its embedded table state, then repeatedly removes records from the collection at offset 0x228. |
| `00cfc0b0` | FormatActivityCategoryPath | speculative | Formats the Activities.Category string using the supplied category identifier and output string buffer. |

### `ui.minigame`

**MiniGameAchievementsDataSource** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c45bb0` | MiniGameAchievementsDataSource_GetColumnName | confirmed | Column-name getter of the trophies table: 0 Id, 1 Name, 2 Description, 3 Icon ID, else '<error>'. |
| `00c46e40` | MiniGameAchievementsDataSource_GetValue | confirmed | Value getter of the trophies table: prefills the out buffer with the wide string '<not found>', resolves the trophy object chain through FUN_008a6760 and FUN_0… |
| `00c47680` | MiniGameAchievementsDataSource_ConstructTrophies | confirmed | Constructor of the trophies variant: registers the instance under the name BaseClient.MiniGame.Trophies (FUN_00922890), stores the arguments at +0x254/+0x258, … |
| `00c45b90` | MiniGameAchievementsDataSource_PopulateOnDemand | likely | Virtual slot 2: when the loaded byte at +0x248 is zero it first calls base vtable slot 15 (offset 0x3c) on the subobject at this-0x30, and then tail-jumps to F… |
| `00c45f80` | MiniGameAchievementsDataSource_PushAllRows | likely | Writes every achievement row into the table sink at +0x34: calls the sink's virtual slot 3 (begin update) once, then for each node of the list at +0x264 (next … |
| `00c472a0` | MiniGameAchievementsDataSource_SetSource | likely | Virtual slot 14: hands the argument to FUN_009b5140 (install the achievements source), clears the populated byte at +0x278 and then invokes virtual slot 4 of t… |
| `00c47720` | MiniGameAchievementsDataSourceA_ScalarDeletingDtor | likely | Virtual slot 0 deleting destructor of the 00c47220 constructor variant: runs that destructor body and frees the object with _free when the low bit of the flag … |
| `00c47220` | — | unknown | Constructor variant of the achievements data source: installs the three MiniGameAchievementsDataSource vtable pointers, runs FUN_00728070 (initialise the achie… |

**MiniGameBossDataSource** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c48040` | MiniGameBossDataSource_GetColumnName | confirmed | Column-name getter of the boss table: index 0 returns Name, index 1 returns Health, anything else '<error>'. |
| `00c488d0` | MiniGameBossDataSource_GetValue | confirmed | Data-source value accessor (vtable slot 4). Column 0 fetches the row's name string through 00836c70; column 1 converts the 32-bit value at row*0x20 + *(this+0x… |
| `00c48e00` | MiniGameBossDataSource::MiniGameBossDataSource | confirmed | Constructor: registers the data source name 'MiniGame.Bosses' through 00922890, installs the three MiniGameBossDataSource base-subobject vtables at +0, +0x30 a… |
| `00c48700` | MiniGameBossDataSource_RemoveAllEntries | likely | When the boss-entry array embedded at this+0x258 is non-empty (count at this+0x260 > 0) it destroys all of its elements by passing that count to the array tail… |
| `00c48930` | MiniGameBossDataSource_AgeEntries | likely | Walks the entry buffer at this+0x25c (count at this+0x260) and for every entry whose float at +0x18 equals the sentinel DAT_01762a30 adds param_2 to the counte… |
| `00c489b0` | MiniGameBossDataSource_RemoveEntry | likely | Linear-searches the entry buffer at this+0x25c (count at this+0x260, stride 0x20 bytes) for the entry whose first dword equals param_2 and, when found, erases … |
| `00c48bd0` | MiniGameBossDataSource_AddOrUpdateEntry | likely | Adds or updates an entry keyed by param_2. It searches with 00c481f0. On a hit it updates the id at +4 and refreshes the name through the lookup service when t… |
| `00c48ea0` | MiniGameBossDataSource_RowCount | likely | Seven-byte getter (bytes 8b 81 30 02 00 00 c3 = mov eax,[ecx+0x230]; ret) that returns the row count field at this+0x230: the element count of the row array ex… |
| `00c48eb0` | MiniGameBossDataSource::~MiniGameBossDataSource | likely | Destructor body: destroys the embedded Entry array (installs its vtable at +0x258, destroys the +0x260 element count through 00c48470, releases the +0x25c buff… |
| `00c48f40` | — | unknown | Scalar deleting destructor (vtable slot 0 of the tertiary base subobject at +0x34): runs the data source destructor 00c48eb0 and frees the object with CRT _fre… |

**MiniGameDataDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c45af0` | MiniGameDataDataSource_GetColumnName | confirmed | Column-name getter of the minigame-definition table: 11 names - Id, Type, Name, Description, Icon ID, Difficulty, Members Only, GroupId, GroupName, GroupIcon, … |
| `00c46c20` | MiniGameDataDataSource_GetValue | confirmed | Value getter of the minigame-definition table: resolves the row through FUN_009ad1b0 and serves 11 columns - 0 id (+0x10), 1 type (+0x14 written through FUN_00… |
| `00c47180` | MiniGameDataDataSource_Destruct | confirmed | Destructor of the minigame-data data source: reinstalls the three MiniGameDataDataSource vtable pointers and runs the base data-source destructor FUN_0091bf60. |
| `00c47210` | MiniGameDataDataSource_GetRowCount | confirmed | Row count of the minigame-definition table: returns the dword at +0x2bf3c of the table object reached through *(this+0x228). |
| `00c475e0` | MiniGameDataDataSource_Construct | confirmed | Constructor of the minigame-data data source: registers it under the name BaseClient.MiniGameData (FUN_00922890), installs the three MiniGameDataDataSource vta… |
| `00c47660` | MiniGameDataDataSource_ScalarDeletingDtor | likely | Virtual slot 0 deleting destructor: runs the minigame-data destructor 00c47180 and frees the object with _free when the low bit of the flag argument is set. |

**MiniGameDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c45a00` | MiniGameDataSource_GetColumnName | confirmed | Column-name getter of the current-minigame table: 17 names - Name, Icon ID, Description, Difficulty, Type, Elapsed Time, Earned Stars, Members Only, Profile Ty… |
| `00c468c0` | MiniGameDataSource_GetValue | confirmed | Value getter of the current/last-minigame row: when no row handle is supplied it reads the current minigame through FUN_009ad100, or the last played one throug… |
| `00c470c0` | MiniGameDataSource_Destruct | confirmed | Destructor of the minigame data source: reinstalls the three MiniGameDataSource vtable pointers, releases the ref-counted goal data source held at +0x25c (virt… |
| `00c474e0` | MiniGameDataSource_Construct | confirmed | Constructor of the live minigame data source: registers it as BaseClient.MiniGame, or as BaseClient.LastMiniGame when the last-flag argument is set; installs t… |
| `00c475c0` | MiniGameDataSource_ScalarDeletingDtor | likely | Virtual slot 0 deleting destructor: runs the minigame data source destructor 00c470c0 and frees the object with _free when the low bit of the flag argument is … |

**MiniGameGoalDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c45760` | MiniGameGoalDataSource_GetColumnName | confirmed | Column-name getter of the goal table data source: index 0 ID, 1 Name, 2 Description, 3 Bonus, 4 Status Text, 5 State, 6 Reward Stars, 7 MembersOnly, 8 Category… |
| `00c45e20` | MiniGameGoalDataSource_GetRowCount | confirmed | Row count of the goal table: resolves the manager through *(this+0x228), asks it for the current minigame with FUN_009ad100(0) and returns that minigame's goal… |
| `00c460e0` | MiniGameGoalDataSource_GetValue | confirmed | Value getter of the goal table: refuses column >= 9 or a missing current minigame (FUN_009ad100(0)); refreshes the index list when byte +0x23c is set (00c45fc0… |
| `00c46f30` | MiniGameGoalDataSource_Destruct | confirmed | Destructor of the goal data source: reinstalls the three MiniGameGoalDataSource vtable pointers, installs the SoeUtil::List<int,-1> vtable for the index list a… |
| `00c472d0` | MiniGameGoalDataSource_Construct | confirmed | Constructor of the goal data source: registers the instance under the data-source name BaseClient.MiniGameGoals (FUN_00922890), installs the three MiniGameGoal… |
| `00c47380` | MiniGameGoalDataSource_ScalarDeletingDtor | likely | Virtual slot 0 deleting destructor: runs the goal data source destructor 00c46f30 and frees the object with _free when the low bit of the flag argument is set. |

**MiniGameGroupDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c45860` | MiniGameGroupDataSource_GetColumnName | confirmed | Column-name getter of the minigame-group table: 32 names from MiniGameId (0) through Price (0x1f), including Name, Description, Icon, LinkName, ShortName, Unlo… |
| `00c47030` | MiniGameGroupDataSource_Destruct | confirmed | Destructor of the group data source: reinstalls the three MiniGameGroupDataSource vtable pointers and runs the base data-source destructor FUN_0091bf60. |
| `00c47440` | MiniGameGroupDataSource_Construct | confirmed | Constructor of the group data source: registers it under the name BaseClient.MiniGameGroup (FUN_00922890), installs the three MiniGameGroupDataSource vtable po… |
| `00c474c0` | MiniGameGroupDataSource_ScalarDeletingDtor | likely | Virtual slot 0 deleting destructor: runs the group data source destructor 00c47030 and frees the object with _free when the low bit of the flag argument is set. |
| `00c463a0` | — | unknown | Value getter of the minigame-group table: rows are param_2*0x70 from the base at *(FUN_009aae50()+0x40); it serves 32 columns - ids/strings straight from the r… |
| `00c470b0` | — | unknown | Row count of the group table: returns the dword at +0x44 of the object obtained from FUN_009aae50 (the group-table singleton). |

**MiniGamePortalDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c477c0` | MiniGamePortalDataSource_GetColumnName | confirmed | Column-name getter of the minigame-portal table: 17 names - TypeId, TypeName, TypeDescription, MiniGameGroupId, IconId, BackgroundIconId, IsMembersOnly, IsMicr… |
| `00c47930` | MiniGamePortalDataSource_GetValue | confirmed | Value getter of the portal table: ensures the rows are populated, rejects row >= base row count or column >= 0x11, resolves the row object (row id from the bas… |
| `00c47d00` | MiniGamePortalDataSource_Construct | confirmed | Constructor of the portal data source: registers it under the name BaseClient.MiniGamePortal (FUN_00922890), installs the three MiniGamePortalDataSource vtable… |
| `00c47910` | MiniGamePortalDataSource_GetRowCount | likely | Virtual slot 2: when the populated byte at +0x22c is clear it first runs the population helper 00c478b0, then tail-jumps to the base implementation FUN_010c8f3… |
| `00c47d90` | MiniGamePortalDataSource_ScalarDeletingDtor | likely | Virtual slot 0 deleting destructor: runs the portal data source destructor 00c47c90 and frees the object with _free when the low bit of the flag argument is se… |
| `00c47c90` | — | unknown | Base constructor body of the portal data source: installs the three MiniGamePortalDataSource vtable pointers and runs the base data-source constructor FUN_0091… |

**MiniGameScoreDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c45800` | MiniGameScoreDataSource_GetColumnName | confirmed | Column-name getter of the score table: 0 Name, 1 Icon ID, 2 Score Type, 3 Score Count, 4 Score Max, 5 Score Points, else '<error>'. |
| `00c45e40` | MiniGameScoreDataSource_GetRowCount | confirmed | Row count of the score table: through FUN_009ad160 (which yields the current minigame's score block) it returns the count dword at +0x70, or 0 when there is no… |
| `00c46fc0` | MiniGameScoreDataSource_Destruct | confirmed | Destructor of the score data source: reinstalls the three MiniGameScoreDataSource vtable pointers and runs the base data-source destructor FUN_0091bf60; nothin… |
| `00c473a0` | MiniGameScoreDataSource_Construct | confirmed | Constructor of the score data source: registers it under the name BaseClient.MiniGameScore (FUN_00922890), installs the three MiniGameScoreDataSource vtable po… |
| `00c462a0` | MiniGameScoreDataSource_GetValue | likely | Value getter of the score table: rejects out-of-range rows against the count at score-block+0x70 (score block reached through FUN_009ad160) and returns 1 after… |
| `00c47420` | MiniGameScoreDataSource_ScalarDeletingDtor | likely | Virtual slot 0 deleting destructor: runs the score data source destructor 00c46fc0 and frees the object with _free when the low bit of the flag argument is set. |

**ShipCombatProcessor** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a6a4b0` | — | unknown | Per-frame UI driver of the Pirates match, gated on the tutorial flag at +0x301 for movie playback (Pirates.log lines Tutorial movie prepared, hiding previous l… |

**(no class)** (32)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009aae90` | MiniGameFlash_EmitOnDisconnect | confirmed | Emits the named event MiniGameFlash:OnDisconnect by calling FUN_009797a0(name, 0, 0). |
| `009aaeb0` | MiniGameToolbarThunk_00c44160 | confirmed | 8-byte adjustor thunk: raw bytes 83 c1 58 e9 a8 92 29 00 decode as add ecx,0x58; jmp FUN_00c44160 - forwards to the minigame toolbar method on the sub-object a… |
| `009aaef0` | MiniGameShowStageSelect_Emit | confirmed | Emits the stage-select events: calls FUN_00704dd0(0, 'MiniGame:ShowStageSelect') and FUN_009797a0('MiniGameStartHandler:showStageSelect', 0, 0). |
| `009b3970` | ShowCollectedMiniGameItemRewards | confirmed | Iterates reward entries, formats item name and quantity, invokes the minigameItemReward UI callback, and builds the displayed collected-items sentence using th… |
| `009b6aa0` | ShowMiniGameHud | confirmed | Switches on active game type and emits HUD hide/show Lua calls; for general games it transfers state flags, sets HUD values and emits HUD:showMinigame. |
| `009b6da0` | StopMiniGameLootWheelAt | confirmed | Finds active state and compares reward entries, computes selected loot-wheel index, sends the ScoreScreen:StopLootWheelAt Lua event with that index. |
| `009b6fe0` | ShowMiniGameMarketplaceAd | confirmed | Builds the HandlerMiniGameStart event name, calls BaseClient.MiniGame.MarketplaceAd, writes UI arguments and emits OnMarketplaceAd through Lua event dispatch. |
| `009b7660` | HandlerMiniGameStart_SetGroupMemberSelectingJob | confirmed | Guarded by this+0xc bit 0x80: looks up the Lua handler through FUN_00977740 with the static name HandlerMiniGameStart (PTR_s_HandlerMiniGameStart_01b468e0) int… |
| `009b7810` | NpxPlayMore_Show | confirmed | Logs the four arguments through NpxPlayMore.log using the format 'Showing play more window. Activity category = %d, image = %s, message = %s, header = %s'; FUN… |
| `009b79d0` | VerifyPlayMoreWindow | confirmed | Logs 'VerifyPlayMoreWindow - category = %d' from this+0x2bef8 and, when that pending category is non-zero and (param_2 is true or the pending flag byte at +0x2… |
| `00a69870` | PiratesShipSelection_ConfirmSelection 🌐 | confirmed | Confirms the ship selection: takes the client object from this+0x14, copies the player name string (client+0x1dc) into a ShipCombatPacketSelectShip temporary b… |
| `00ad3b40` | ReleaseTcgPointer | confirmed | Looks up TcgMainWindow, verifies GuiTradingCardCtrl type, and calls ReleasePointerCapture. |
| `00ad3fd0` | CreateTcgWindow | confirmed | Finds GameScene, casts it to GCtrl, creates and configures a TcgMainWindow control, binds current manager state, inserts the control and initializes its geomet… |
| `00c07e50` | RegisterMiniGameModule | confirmed | Registration wrapper: pushes nup=0, the 11-entry table 0x0182a100 and the 'MiniGame' module-name pointer, calls the Lua module registrar FUN_00e39e40 and retur… |
| `00c08280` | RegisterMiniGameFlashCModule | confirmed | Registration wrapper: pushes nup=0, the 7-entry table 0x0182a1e8 and the 'MiniGameFlashC' module name, calls FUN_00e39e40 and returns 0. |
| `00c08570` | MiniGameFlash_RequestTintByName | confirmed | Lua 'RequestTintByName' handler: with at least one argument it resolves the target object from the argument (FUN_00796170, then FUN_00795ce0 for the object han… |
| `009aaec0` | AdjustorThunk_add58_009aaec0 | likely | 8-byte adjustor thunk: raw bytes 83 c1 58 e9 a8 91 29 00 decode as add ecx,0x58; jmp FUN_00c44070 - forwards to a minigame toolbar method on the sub-object at … |
| `009aaed0` | AdjustorThunk_add58_009aaed0 | likely | 8-byte adjustor thunk: raw bytes 83 c1 58 e9 a8 8e 29 00 decode as add ecx,0x58; jmp FUN_00c43d80 - forwards to a minigame toolbar method on the sub-object at … |
| `009aaee0` | AdjustorThunk_add58_009aaee0 | likely | 8-byte adjustor thunk: raw bytes 83 c1 58 e9 18 8f 29 00 decode as add ecx,0x58; jmp FUN_00c43e00 - forwards to a minigame toolbar method on the sub-object at … |
| `009b3c30` | PreparePlayMorePrompt | likely | Looks up activity and category metadata, formats NpxPlayMoreDescription and NpxPlayMoreHeader, logs the value at manager+0x2bef8, and when that field is zero w… |
| `00c082d0` | MiniGameFlash_LoadData | likely | Lua 'LoadData' handler: with at least one argument it takes the last value as a string, assigns it into a stack SoeUtil::IString (FUN_00402ce0) and calls FUN_0… |
| `00c45e60` | MarkAttachedSourceDirtyAndNotify | likely | Marks the attached goal data source dirty (sets byte +0x26c of the object at this+0x25c when present) and then tail-jumps to virtual slot 4 of this object (byt… |
| `00c460c0` | FormatStateValueIntoBuffer | likely | Formats the dword at *param_2 into the buffer param_1 through FUN_0082ecd0 with the format object stored at DAT_017703e0 and returns the produced string length… |
| `00c478b0` | PortalRowSet_PushAllToSink | likely | Pushes every portal row into the table sink at +0x34: iterates the portal rows with FUN_009acf60 / FUN_009acf70 and, for each row whose dword at +0x24 is non-z… |
| `00c481f0` | EntryArray_FindByKey_Stride0x20 | likely | Linear scan of an entry array embedded at this+0x25c with the count at this+0x260: compares each 0x20-byte entry's first dword to the key argument, advancing 8… |
| `00a69980` | PiratesShipSelection_SelectAndConfirm 🌐 | speculative | Ship-selection confirm with an explicit index: stores param_2 at this+0x36c (the chosen ship index), scans the ship chain at client+0x294 to that index and cop… |
| `00ad3ae0` | CaptureTcgPointer | speculative | Looks up TcgMainWindow, verifies the control is GuiTradingCardCtrl, then calls SetPointerCapture with zero. |
| `00c45fc0` | IndexList_RebuildFromRecordArray | speculative | Rebuilds the sorted goal-index list of the data source: it iterates the minigame goal array (base *(miniGame+0x80), count +0x84, stride 0x110) and, when the go… |
| `009b7170` | — | unknown | Selects active state or searches a state bucket by id and suppresses duplicate result display. For each 0x110-byte objective record, status 6 is skipped; when … |
| `009b9fc0` | — | unknown | Logs 'MiniGame:GroupInfo', copies the decoded group record into the manager's +0x2bea8..+0x2bee0 fields through FUN_009b8bc0, and publishes each field to the U… |
| `00ad3a60` | — | unknown | Looks up TcgMainWindow, casts it to GuiTradingCardCtrl, then invokes control slots +0xc and +0x10 with zero and releases the temporary control reference. |
| `00c08480` | — | unknown | Lua 'SendMessage' handler: with at least one argument, when the client object's +0xf8 field is set, copies the string argument into the global MiniGameFlash me… |

## Source shards

[[S0220 008a3ce0-008a8500|S0220]] (2), [[S0224 008bcf60-008c5290|S0224]] (1), [[S0245 0092b170-0092f380|S0245]] (1), [[S0247 00934740-00939200|S0247]] (1), [[S0269 009a6cf0-009aceb0|S0269]] (37), [[S0270 009acf20-009b2140|S0270]] (28), [[S0271 009b2210-009b7170|S0271]] (19), [[S0272 009b7660-009bd590|S0272]] (31), [[S0273 009bd650-009c2c50|S0273]] (38), [[S0274 009c2cc0-009c8d80|S0274]] (34), [[S0283 009f39b0-009f8370|S0283]] (1), [[S0303 00a674c0-00a6c3b0|S0303]] (3), [[S0322 00acd6d0-00ad3250|S0322]] (3), [[S0323 00ad3270-00ad83b0|S0323]] (18), [[S0324 00ad8880-00adee10|S0324]] (25), [[S0329|S0329]] (4), [[S0356 00b80ac0-00b84ce0|S0356]] (4), [[S0366 00bb11b0-00bb7710|S0366]] (25), [[S0374 00bd9880-00be0090|S0374]] (8), [[S0375 00be0120-00be58f0|S0375]] (1), [[S0376 00be5b30-00bea020|S0376]] (9), [[S0383 00c05780-00c0db00|S0383]] (5), [[S0392 00c42400-00c482e0|S0392]] (55), [[S0393 00c48300-00c4e490|S0393]] (51), [[S0394|S0394]] (1), [[S0410 00ca6401-00cab4e0|S0410]] (4), [[S0418 00cd1210-00cd5dd0|S0418]] (1), [[S0420 00cdb370-00ce0140|S0420]] (6), [[S0422 00ce4e30-00ceb920|S0422]] (34), [[S0425 00cf69c0-00cfc480|S0425]] (14), [[S0426 00cfc4e0-00d01c70|S0426]] (12), [[S0580 0107d0c0-010829c0|S0580]] (9), [[S0581|S0581]] (12), [[S0683|S0683]] (2)
