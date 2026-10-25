---
type: build-guide
section: Gameplay
updated: 2026-10-12
---

# Trading Card Game

Part of [[Build Guides]] › Gameplay. Related: [[Minigame Framework]], [[Coin Store and Purchases]].

<!-- NARRATIVE START -->
## How it works

**How to read this.** Confidence words (`confirmed`, `likely`, `speculative`) are the native-map labels for a function. Source tags: *client* = read from the shipped client in this pass or in a shard (function pseudocode and bytes, shipped files in the client baseline's `Resources`, `UI` and `tcg` folders, the decoded Lua `ScriptsBase.lua`, the client locale); *client Lua* and *client locale* say which of those; *video* = a reviewed frame record (`video/frame` keys, player names and chat never recorded); *wiki* = the fan wiki archive; *vault* = an earlier research note, used only for facts it took from the client (the mini-game tables and locale ranges in Piece 010, event text in Piece 019); *R* = reconstructed, our own server policy and not original behaviour. Precedence is client, then video, then wiki. **Not used:** the community packet catalogue, any emulator or legacy-server code, and `tcg/frtcg.ini` (not opened). **Not analysed:** `tcg/FreeRealmsTCG.dll` and everything it does on its own. Where the DLL would decide an answer, the text says so instead of guessing. Related: [[FRTW Game]] (the Treasure War display layer), [[Card Duelist]] (jobs, quests, NPC opponents), [[Minigame Framework]], [[Matchmaking and Lobby]], [[Coin Store and Purchases]], [[Items and Inventory]], [[Trade]], [[Social (Friends, Guilds, Chat)]], [[Quests and Objectives]].

### Overview

**The first thing to know: "the card game" is two different games, and the mapped functions are mostly the second one.**

| | Free Realms Trading Card Game ("TCG") | Treasure War |
|---|---|---|
| Mini-game rows | `MiniGameData.txt` TYPE 16: 124 rows (lobby 499, tutorial 537, Kai's challenge 1036, NPC duel rows 387 to 710 and others) plus TYPE 3 row 11 "FR Trading Card Game". *client file + client locale* | TYPE 23, row 727 "Treasure War". *client file + client locale* |
| Where it runs | A separate Qt 4 module, `tcg/FreeRealmsTCG.dll`, loaded and composited by the exe. *client* | Inside `FreeRealms.exe`, on an embedded engine with Scaleform movies (`TCCW_*.swf`). *client* |
| Launcher object | `ClientTradingCardGameManager` (ctor `00ad6300`, startup `00ad47f0`) | `ClientSimpleCardGameManager` (ctor `00ad1590`, init `00ad2d30`) |
| How the client chooses | `DispatchMiniGamePacket` `009bf890` looks the mini-game state up by id; state type 16 goes to the first manager, type 23 to the second. *client pseudocode* | same |
| What this guide maps | Only the launch, window, input and portrait glue. The DLL itself is **not mapped**. | Nearly all 3,412 functions: the card engine, rules, commands, deck, collection, tournament and Scaleform action layers. |
| Platforms | Windows. The Lua carries a "no TCG on Apple" message (`ShowNoTcgOnAppleMessage`) and sends Apple clients to Treasure War. *client Lua* | Also offered on Apple clients: the card-games button calls `StartTreasureWar` when `Ui.IsRunningOnMac()`. *client Lua* |

Both games appear to share one lineage. The exe's own source-path strings read `c:\dev\p4\FreeRealms\Main\Code\External\TradingCardsClient\common\...`, and the DLL's folder ships the same data layout the exe code expects (`data/archetypes`, `data/collections`, `data/decks`, `data/tutorial`), which is why we read them as one family (inference for the DLL side). Names left in the shared code show its older life: the card client window title is "Clone Wars: Trading Card Game" (`00f9d320`), `TradingCardAssets.txt` lists 40 Clone Wars character thumbnails, and class prefixes are `CTP::Game` (base engine), `CW*` (rules game and nodes), `ZP*` (older names in assert strings), `FRTW*` (Free Realms Treasure War handlers). *client*

**What a player sees**

- **Entry points.** The nav-bar card button and game-dock button 9 open a utility-window TCG panel (`ShowTCGPanel`) whose callbacks are `gotoTCG` (calls `TradingCardGame.StartTutorial()`) and `gotoCardBattles` (calls `StartTreasureWar()`); the welcome screen "TCGLobby" link reaches `StartTutorial` too, as does equipping a card-game store item: the store reply carries `tcgItemEquipped` and `twItemEquipped` flags and the Lua launches the matching game. *client Lua, `ScriptsBase.lua`* Video shows the "Select Game" panel with both names (`ip2vKMeCU6U/00062`, `AW041GUrkus/00104`) and purple "Trading Card Game" world markers. *video*
- **TCG lobby.** The mini-game row text reads: learn to play, practice against Kai in a single-player match, take on other Card Duelists, check the collection, trade, build decks. *client locale* Video shows the "Free Realms Trading Card Game Lobby" with a "Casual Games Lobby" list (Games, Players, Friends, Waiting for Opponent, Friend Only, Timed Game), a welcome panel (membership perks, "Create and accept trades", events and leaderboards, Learn to Play, Build a Deck, Play a Game, Card Duelist Job, a "Card Hall" icon to move around), a trophy icon for tournaments, an "Exit TCG" confirmation, and a loading panel "Please wait while we load the Free Realms Trading Card Game / Waiting for server...". *video* (`UFsVcZ3X5iA/00074`, `4h2eVoje3Eg/00060`, `4h2eVoje3Eg/00070`, `4pioItLqlU8/00118`, `jfcerhMCY6g/00022`, `UFsVcZ3X5iA/00100`)
- **Rules as shown.** "Be the first to score 12 points to win the match" appears in 116 of the 124 TYPE 16 row descriptions and in the shipped campaign file; most rows also name the card the winner takes from the opponent's deck. *client file + client locale* Cards belong to four spheres, Machine, Chaos, Nature and Order, and decks are 40 cards ("This 40-card starter deck", "Build up your score piles to unlock the special powers of your cards"). *client locale + video* The result screen lists Number of Turns, Margin of Victory, Cards Played, Heroes Played, Biggest Combat Differential, Tricks and Resources Played, cards by sphere, Cards Destroyed. *video* (`iud_NyVa_2o/05176`)
- **NPC duels.** Each TYPE 16 duel row is one opponent deck with a difficulty of 1 to 5 (counts 41, 25, 39, 8 and 10; lobby row -1). There are 44 "Round" rows (First to Fourth Round for nine named opponents, two of them with eight rows), 68 rows with "Deck" in the name (48 follow the "<NPC>'s Trick, Creature, Resource or Tournament Deck" pattern, 12 each), three Player's Club "Battle" rows, and 9 others: the lobby, the tutorial, a Halloween row, Kai's challenge and 5 unnamed rows (700 to 703 and 9997). *client file + client locale* Opponent names, places and prize cards are content: see [[Card Duelist]].
- **Collection, packs, rewards.** The collection window has Cards, Packs and Virtual Rewards tabs, search by Cost, Attack/Defense and Rarity, "Show Trade Limits", Redemption and Delivery windows. A virtual reward card redeems into one item for one character and disappears from the collection. *video* (`UFsVcZ3X5iA/00028`, `4pioItLqlU8/00132`, `jfcerhMCY6g/00028`, `eS3bRs9fR5k/00020`) Pack products in the client locale: Set 1 (Original/Premiere), Set 2 Shattered Past, Set 3 Baron von Darkcheat, Set 4 Heroes, Set 5 Rumbledome, Treasure War Booster Pack, 25-pack bundles, promotional sample packs, four-sphere starter decks for Sets 2 and 3, "Loot Cards". *client locale* A pack is "10 game cards and 1 virtual reward card". *client locale + video + wiki*. Members get "a TCG booster pack each month" and can "create and accept trades". *video + wiki*
- **Events and tournaments.** An event calendar lists timed sessions (Treasure War events 3:30 to 4:30 AM, 4:45 to 5:45 AM, 6:15 to 7:15 AM on one captured day) with "Earn Gold and Promo Cards!"; a leaderboard and organized-play rules exist for both games. The client rules text says the first three games each event day are "Top Duelist Games", 3 points for a win and 1 for a loss, tie-breakers from the wins of that day, ranking over 14 calendar days, and no prizes for Top Duelist Games. *video `JYoSoRzyLrI/00118`, client locale, vault 019*
- **Treasure War.** Lobby buttons: Quickplay, Campaign, Tournament, Collection, Buy Cards. The duel screen states the rules: red beats green, green beats blue, blue beats red, equal colours go to the higher power, win the duel then take a treasure, take four treasures to win. Cards show a type line (Dwarf, Human, Pixie, Animal, Chugawug, Changeling), a power number and game text; the collection filters power 0 to 12 and has a "View Gems" button ("Double click a card to bind a gem to it"). *video* (`JYoSoRzyLrI/00164`, `00044`, `00290`, `00414`) The wiki adds that gems keep a card in the deck or hand for a number of games and that decks are drawn at random from owned cards. *wiki Treasure War*
- **Progression and shop.** Card Duelist is the job; its level-up uses badge image set 1783 mapped to `levelup_tcg` in the Lua, and the quest line, trainers and prize cards are in [[Card Duelist]]. *client Lua* Packs are sold in Station Cash Marketplace category 83 (`MarketplaceStoreCategoryEnum.TCG`), hidden on Apple clients; a separate "Treasure War" category exists in the store listings. *client Lua + video*

### How the client implements it

#### Main classes

| Class / object | Role | Evidence |
|---|---|---|
| Lua module `TradingCardGame` (`RegisterTradingCardGameBindings` `00c04950`) | Three entries: `StartTutorial` (`00c04970`), `StartTreasureWar` (`00c049f0`), `Shutdown` (`00c04920`, not mapped). The two starters walk a list on the mini-game manager (offset `0x638`, key `0x6d54a188`; offset `0x12d8`, key `0xbfd77cb0`) and call the activity-join sender `00be7d80`. So "play" is a request to the server to join an activity; the server then opens the mini-game state. | confirmed `00c04950`; `00c04970` and `00c049f0` unlabelled (pseudocode read) |
| Lua handler `TradingCardGameHandler` (window `Main.wndTradingCardGame`, mini-game z-layer) | `OnCampaign/Tournament/Collection/MatchmakingLoadStart/Complete` show and hide the asset-loading panel; `OnLoadFail` hides it, runs `gotoPortal` (`Ui.CreateMiniGameGroup(57, 0)`) and shows the string `ui_ccg_ServerError`; `quitGame` calls `TradingCardGame.Shutdown()`. | client Lua |
| `DispatchMiniGamePacket` (`009bf890`) | Family 39 sub-op router for this feature: `0x28` start game, `0x29` load failure; other sub-ops go to the generic mini-game paths. | confirmed |
| `ClientSimpleCardGameManager` (ctor `00ad1590`, `InitCardClient` `00ad2d30`, state machine `00ad2380`, `HandleTcgLaunchTicket` `00ad1a80`, `Shutdown` `00ad1ef0`) | Treasure War host. Creates the TCG interface and Flash view, loads `UI/TradingCard/TCCW_Main.swf` (movie `TradingCardMain`), logs realm, session ticket, launch ticket and station name, hides HUD, dock and chat. | confirmed |
| `ClientTradingCardGameManager` (ctor `00ad6300`, `InitializeAssets` `00ad3670`, `Startup` `00ad47f0`, `Update` `00ad4f50`, `OnDisconnect` `00ad3740`, `Shutdown` `00ad3e40`) | TCG (DLL) host. Starts the `AssetsTcg` download, builds the launch arguments, loads the DLL, relays window, mouse, key, music and portrait traffic. | `00ad47f0` unlabelled; the rest confirmed |
| `TCGInterface` / `ClientTCGInterface` (`00ad1640`; 13 + 3 mapped functions) | Adapter object through which the card engine reads the exe's globals (account pair, flags, handlers). | confirmed |
| `TradingCardGame::Manager` (`TcgManagerSingletonCtor` `010d25f0`), `GuiTradingCardCtrl` (ctor `010cee90`, 28 functions), window tracker `010d1690` | Singleton that owns the DLL, its argument strings and a list of up to 256 DLL windows; the GUI control named `TcgMainWindow` draws the DLL's surfaces and feeds mouse and key input back (key map `010ce4f0` turns X11-style keysyms into GUI keys). | likely / unknown labels |
| Card client (`CardClientInit` `00f9d320`, source `cardclient.cc`) | In-exe card client start-up: parses `--lang --sessionID --realm --fullscreen --username`, reads `preferences.dat` and `/debug/debug_mode`, maps the realm to one of five environments, creates five services (Streaming, UI, Input, Scaleform, Audio). | likely |
| `Service::ServiceManager` (`012e0ba0`) and `Messaging::UILoop` (`012e5730`, push `012e5820`) | Process-wide service registry; the single queue that carries UI actions to Flash. | likely |
| Handler families (`ui.tcg`, `ui.deck`) | `IScaleformActionHandler` classes (UI to engine: `Engine::Deck*`, `Collection*`, `Tournament*`, `FRTW*`) and `IDisplayActionHandler` classes (engine to UI). Each has ctor, deleting dtor, action-name slot and trace label that leaks the original file and line (`tournamenthandlers.cc`, `deckhandlers.cc`, `collectionhandlers.cc`, `networkhandlers.cc`, `frtwgamehandlers.cc`, `scaleformhandlers.cc`, `frtwcampaignhandlers.cc`). | confirmed (S0654 to S0665) |
| `NetworkService` (`012f1890`, framing `012f0a00`, inbound `012f2150`, host file `012f1b80`) and `LoginService` (`012efb70`) | The card client's own TCP command connection. See Network. | likely |
| `CTP::Game` -> `CWGame` -> `CWMainController` (ctor `01314350`, class id 140003) | The match object. `CTP::Game` holds players, observers, turns, play elements, the command table and the event dispatcher; `CWGame` adds setup, win/draw/adjourn and the display-action drain; `CWMainController` adds the single-player session start, AI controllers and challenge status. | confirmed |
| `Player` (id 261), `PlayElement` (167), `PlayArea` / `PilePlayArea` / `PlayerPlayArea` (172 to 174), `Card` (168) / `CWCard` (140001) | Serializable match objects. A player owns draw, discard and hand piles; a card has an archetype id, in-play and being-discarded flags, property maps, actions and keyword lists. | confirmed |
| `GameTurn` (232) / `CWGameTurn` (140004) | Turn and phase tracker (cards drawn per turn, skipped phases, phase-played map). | confirmed |
| State machine (`StateMachineState` id 234 and about 25 subclasses) | `HandleResponseState`, `PlayCardState`, `PlaySingleCardState`, `DrawCardState`, `DiscardCardState`, `RevealState`, `ReshuffleState`, `MultiActionState`, `GetTopCardState`, `GetSimultaneousTargetState`, `ExecuteRuleState`, `DoublePassState`, `CustomChoiceDialogState`, `CloneState`, `ChainedActionState`, `IntroduceCardState`, `ProcessEventEmissionState`, `PhaseState` and CW variants. | confirmed |
| Rule nodes (`ExpressionTreeNode` 39 and about 45 subclasses) | Data-driven card rules: `ActionNode` (44), `ActionPlayNode` (49), `QueryNode` (41), `PileQueryNode` (76), `MoveActionNode` (115), `SetEnvironmentNode` (74), `EmitTextMessageNode` (48), arithmetic and comparison expressions (`Sum` 368, `Each` 35, `IfThen` 28, `Repeat` 367 ...), `CWMacroNode`, `CWTargetQueryNode`. | confirmed |
| Command objects (`CommandObject` 178 and 58 further classes, ids up to 472) | The results of rule evaluation that change the game: `Move`, `DrawCard`, `Discard`, `Shuffle`, `Reveal`, `PlayCard`, `AddKeyword`, `SetProperty`, `ModifyProperty`, `InstallAction`, `Prevent`, `Kill`, `WinGame`, `LoseGame`, `PickList`, `MultiAction`, `CreateCard`, `Clone` ... Each has do and undo. | confirmed (S0786 to S0789) |
| `EvaluationEnvironment` (170), `AttributeModifier` (169), `HandleResponseState` push-class table (`014127b0`) | Evaluation context for a rule; modifiers that change card numbers; the table by which a state names the next state by id. | confirmed / likely |
| Local databases | `ArchetypeDB` (`012b8f80`), `ActionDB` (`012bdd60`), `MessageDB` (`012a84b0`), `CollectionDB` (`012adee0`), `DeckDB`, `TutorialDB` (`01406d50`) over `DirDBM` stores, plus the binary object database `odb` (`01511ee0`, `015134a0`). | confirmed / likely |
| `CWAIController` (`handleScript` `014050d0`, `getInput` `014059d0`) | Client-side AI for single-player games. | confirmed |

#### Launching a game (both games)

1. **Request.** The player's click reaches `TradingCardGame.StartTutorial/StartTreasureWar` (or the equipped-item path), which asks the server to join the activity (`00be7d80`). The challenge request `RequestTCGChallengePacket` (family 39 sub-op `0x26`, ctor `009a9ec0`, sender `009bdaf0`, one id in the state slot) also exists; no static caller of `009bdaf0` was found, so what triggers it is open. *client*
2. **State.** The server creates the mini-game state. The state's type decides the manager: 23 to the Simple Card manager, 16 to the Trading Card manager (by the object offsets `+0x108` and `+0x104`, which `PlayClient::Initialize` `0092b170` fills with the two manager objects). *client pseudocode, `009bf890`; likely*
3. **Start message.** The server sends `TradingCardStartGamePacket` (39/`0x28`). For type 16 the client calls manager slot 25 (`00ad32e0`, stores two ints at `+0x2ba98/+0x2ba9c`, which the startup log prints as one 64-bit "Launch Ticket"). For type 23 it calls slot 27 (`HandleTcgLaunchTicket` `00ad1a80`) with the two ints and a third value taken from a header id slot. *client pseudocode*
4. **Failure.** Sub-op `0x29` runs Lua `TradingCardGameHandler:OnLoadFail`: the loading panel hides, the player returns to the portal and sees the server-error string. *client*
5. **Treasure War state machine** (`00ad2380`, field `+0x2ba90`): state 1 loads `SimpleCardAssets.txt` through the asset system; state 2 waits until more than two time units have passed and a pending-load field is clear, then sends family-39 sub-op `0x25` (`009bf590`) and enters state 5; state 5 waits for the launch ticket and, once a timeout threshold passes, calls the launch handler with zeros each update, which leaves the state unchanged. The handler accepts a ticket **only in state 5** (otherwise it logs "Received TCG launch ticket but state is %d"); a nonzero ticket moves to state 3 and restarts the clock, and a positive third value records mode 1 plus that id and fires Lua `OnCampaignLoadStart`. State 3 waits for the card client to be ready and sends it `goCampaign(id)`, `goTournament`, `goCollectionManager` or `goMatchmaking` for mode 1 to 4; state 4 fires the matching `On...LoadComplete`; when the card client closes, the manager sends family-39 sub-op 6 (labelled leave, `009be8e0`) and, for modes other than campaign, Lua `gotoPortal`. *client pseudocode* Reading of the third value as a campaign mission id is an inference from state 3 and 4 (likely).

#### The TCG (DLL) boundary

- **Files.** `tcg/` holds `FreeRealmsTCG.dll`, the Qt 4 DLLs (Core, Gui, Network, Script, Svg, Xml), the VC8 runtime, `cards.rcc` (about 51 MB), `core_resources.rcc`, `resources.rcc`, `effects.rcc`, `sounds.rcc`, `foil.rcc`, `client.*.qm` translations, `locale/*_data.dat/.dir`, `data/`, `AssetsTcgW_000.pack` and `AssetsTcg_manifest.txt`. *client files* The exe asks the client-settings key `AssetDelivery/TcgServerAddress` (value `http://fr.patch.station.sony.com/patch/freerealmstcg/live/client` in `ClientConfig.ini`) for patches; `InitializeAssets` starts an asset resource set named "Free Realms TCG". *client file + client pseudocode*
- **Loading.** `010d1ab0` loads `FreeRealmsTCG.dll` from the manager's directory (log "Failed to load tcg module" on failure) and binds 21 exports, activating the module only if all are present: `Initialize`, `RunFrame`, `Shutdown`, `GetWindows`, `GetCaptureWindow`, `GetCurrentCursor`, `OnWindowStateChanged`, `OnMusicCompletion`, `GetWindowRepaintRects`, `GetWindowSurfaceData`, `OnMouseEvent`, `OnMouseWheelEvent`, `OnKeyEvent`, `OnFocus`, `SetAvatarImage`, `SetPortraitImage`, `OnMMODisconnect`, `OnMMOSystemMessage`, `OnSoundCompletion`, `OnMMOSoundLoaded`, `SetVolume`. *client*
- **Launch arguments.** The manager builds `--lang= --country= --username= --realm= --sessionID= --character= --challenge= --ticket=%llu --host=fr --is-founder=true|false --post-login=tutorial` (producer `00ad47f0` and its helpers `010d2120` to `010d2450`). The DLL therefore receives its own session identity and likely talks to a Sony card server itself. *client; "likely talks" is inference*
- **Display.** The exe does not draw the DLL's UI itself: it asks `GetWindows`, `GetWindowSurfaceData` and `GetWindowRepaintRects` each frame, composites the surfaces into `TcgMainWindow`, and forwards mouse, wheel, key, focus and window-state events. Avatars and portraits are pushed in with `SetAvatarImage` / `SetPortraitImage`; music and sound completion are called back. *client, S0595*
- **Portraits.** `ScreenshotManager::CaptureTCGPortraitToBuffer` (`00b4e9f0`) renders a character into a 0x33c by 0x3ba buffer (828 by 954) for card art; the portrait job processor `00b50190` chooses between that local capture and a server portrait request (`PacketGeneratePortraitRequest`, family BaseFotomat sub-op 1, ctor `00ad3190`, tunneled by `00ad61f0`). The locale offers "custom card packs with your picture". Linking the two is inference. *client + client locale*
- **Everything else is unknown.** What the DLL sends, which server it contacts, how it reads `tcg/data`, how NPC duels are played and how rules, scoring and the richer end-of-game statistics (video shows Heroes Played, Biggest Combat Differential) are implemented are not recovered. The exe-side `GameData` keeps only four statistics (result, turns, cards played, cards destroyed; `01428f50`). *client*

#### The in-exe card client (Treasure War)

**Boot.** `InitCardClient` (`00ad2d30` to `00f9d320`) creates the services, then the Scaleform bridge: Flash calls `RequestSessionIDLogin` / `SessionID` / `RequestEffects` / `RequestAnimations` / `CloseClient` go through named actions; the Flash movie root is `_root.mainWindow` and the engine calls `ChangeScreen(ScreenName, GroupID)` and `Dispatch` on it (S0652). Window size defaults to 800 by 600. The card client's realm maps `frscg_live`/`production` to environment 0, `frscg_main` (and the default) to 1, `frscg_qa` to 2, `frscg_stage` to 3, `frscg_livetest` to 4. *client*

**Login and screens.** After the command connection is up (see Network) the client sends a session-id login. `LoginResultsDisplayAction` (`0124f780`) reads success and a result code; code `0x4c` goes to user-name registration, success queues `PostLogin`, `GoTo`, `GameScreenLoaded` with guard `goMenu`, failure queues `CloseClient`. Screens exist as Scaleform movies: Home, Campaign, Collection, Tournaments, Profile, Game, Delivery, Redemption, Rewards List, Imbue Holocrons, Treasure Trove, Event Calendar, Tournament Rules, Friends List, prompts. *client* (`TradingCardAssets.txt`, `Assets_manifest.txt`)

**One match, as the engine sees it** (all *client*; steps 3 to 8 are the shared `CTP::Game` engine and read as server-authoritative because commands carry a game id and a "Server ignoring game commands" guard exists):

1. **Lobby and match.** `TradingCard::Lobby` (command 103, v3) is a lobby snapshot: five child lists, five role sets (General, Player, Observer, Moderator, Staff), group ids, lobby type, a deck object (v2+) and tournament round and sub-state (v3+). `LobbyCommand_Join/Leave`, `MatchCommand_SetReady/SelectDeck/SelectStartPosition/SetBid/SetDeckArchetype/SetTimer/SetMatchState/InitiateChallenge` set a match up; `MatchCommand_LaunchGame` (116) makes the client create its game object (0x40c bytes, `014596b0`). *likely*
2. **Setup.** `GameCommand_SetupGame` (67, v10) carries game id, server id, player count and order, ordered account ids, game version, match id, duration, game type, play type, flags, game number, league id and match structure; a version mismatch only logs a display action 0x14 and applies anyway (`014d7a90`). `CWGame::Setup` (`01336d20`) builds two 0x9cc-byte objects and one `CWPlayer` per player. `GameCommand_SelectDeckForPlayer` (58) and the deck services supply decks; `SendSerializedGame` (262) delivers a zlib snapshot for resync or join (`014d9550`).
3. **Start gate.** `AdvanceTurn` (`01338880`) on the first call creates the `CWGameTurn` and submits `GameCommand_ReadyForStartOfGame` (117). The submit gate (`0136c400`) deletes every other command type while its `+0x14e` flag is set, and the queue flush (`0136ea70`) frames queued commands only once the started flag `+0x118` is 1.
4. **Play.** Client input becomes `GameCommand_*` objects through `DispatchCommandObject` (`0136d640`): `CardPlayed`, `BeginCardPlayed`, `CardIntroduced` (`IntroduceCard`), `ActionPlayed`, `CardSelected`/`CardDeSelected`, `ButtonPressed`, `StateSpecificMessage`, `DragStopped`, `SynchPoint`, `DiscardCard`, `FinishCustomChoice`, `PrePass`. Event codes in `CTP::Game`: 1 cardPlayed, 2 cardIntroduced, 4 actionPlayed, 5 cardSelected, 7 playAreaSelected, 8 tokensMoved, `0xb` buttonPressed, `0xc` buttonPressedWithText, `0xd` stateSpecificMessage, `0xe` setup-deck player, `0xf` dragStopped, `0x10` synchPointReached. A text form of the same events (`cardPlayed(...)`, `receiveList(...)`, `concede(...)`) is parsed by `DispatchTextCommand` (`01337510`, `01369c70`); the channel that feeds it is unknown.
5. **Rule evaluation.** The current state decides what input is legal (button handlers check the player id: "Got an buttonPressed from %d instead of %d"). Card rules are node trees evaluated into command objects; `EmitTextMessageNode` and `ActionPlayNode` evaluate to `CommandObject_EmitTextMessage` / `Reveal`, so evaluating a card produces command traffic. The server names the next state by id (`HandleResponseState::GetPushClass` `014127b0` maps 18 ids) and the client builds it. Random draws use a standard MT19937 (`01419a30`, bounded draw `01419f70`); the deck shuffle is a two-pass Fisher-Yates unless the `noshuffle` option is set (`0139f3a0`).
6. **Timeouts and offers.** Slot 24 (`0136e320`) warns two minutes before the timeout, then submits `GameCommand_RemovePlayer` (reason 2); slot 23 (`0136e780`) enforces the per-turn limit (`+0x244`). `Concede` (`013734b0`) submits `ConcedeGame` (reason 1); draw and adjourn offers record per-player votes, submit `AcceptDraw` (161) / `AcceptAdjournment` (263) and call `EndAsDraw` (`013358f0`) / `EndAsAdjournment` (`01335a10`) when all accept.
7. **End.** The win check runs on the client: `SetGameWonByPlayer` (`01363270`) finds the winner in the player list and posts the win or defeat text; nothing in the code shows a server winner announcement. `MatchWonByGameCount` (`01331120`) is true when a player has 2 wins (match flag 2) or 3 (flag 3), a best-of-three or best-of-five reading from the thresholds alone. `DeliverGameEndNotice` (`01332ed0`) builds surrender, give-up, time-out and countdown notices. `GameCommand_GameOverReached` (383), `EndGame` (471) and `LostPlayer` (139) exist.
8. **Faults.** `OutOfSync` (64), `FatalError` (132, carries the exception text and a zlib-compressed game state) and `HandleCaughtGameError` (`0136d810`, called from 56 catch funclets) report errors; recoverable per-player errors are logged and play continues.

**Single-player and campaign.** `CWMainController` sends `AccountCommand_RecordSinglePlayerGame` (487: activity 1 begin or 2 end, game type 1 tutorial, 2 scenario, 3 skirmish, result 1 win or 2 lose) and `SetPreferences` (118) at start, and `AccountCommand_SetChallengeStatus` (488: challenge id and result 1, -1 or 0) at the end and in its destructor. The AI deck selector at `+0x107` divides by 10000 (2 "Digital Starter - Machine", 3 "... Nature", 4 "USE AI"). Tutorials replay scripted rule sets from `TutorialDB` (`data/tutorial`: `version`, `initial_values`, `deck__1`, `deck__2`, `rule__%d`, each a compressed blob). The campaign screen (`0131faa0`) builds 8 campaign entries (`CampaignID`, `ProgressionEnabled`, `Purchased`, `Cost`, `CampaignXML`) plus boss unlocks and a `GoToStage` event; `AccountCommand_SetCampaignStatus` (415) reports status. The embedded campaign table (`01329630`) holds 74 entries: titles "TW 1.01" to "TW 7.10" (ten per chapter, seven chapters) and "TW B.01" to "TW B.04", each with a display name (for example "Cruisin' for a Bruisin'", "The Eye of Necrosis", "Queen in Peril", "Kart Racing Champion"), a `campaignImage_NNN` asset (the 74 images in `SimpleCardAssets.txt` are 101 to 110 through 701 to 710 and 801 to 804) and a `Tcg_Opponent.nameNNN` locale key. *client* A second path (`BuildTestCampaigns` `012510d0`) builds 40 placeholder missions ("TEST CAMPAIGN TITLE"); which path ships is open.

**Decks, collection and trading.** Deck formats are ids 1 (Standard), `0x1adb1` (Standard Draft), `0x1adb2` (Highlander), `0x1adb3` (Common Ground) and `0x1adb5` (No Rares), checked by `CWDeckBuilderUtilities::ValidateFormat` (`013ed580`): at least 40 cards in the draw deck (`013eb7b0`, message "You must have at least %d cards in your draw deck."), copy limits 3 and 1, a "cards only from %s" collection rule; return values are unproven in places (S0735). Deck services validate to "Invalid Deck!" / "Read-Only Deck!" (`013aa220`). Deck actions from the UI: query, new, open, save, delete, load, select, add/remove filter, reset filter, sort (four modes), request the deck collection; `SetSearch` and `RequestTickets` are `return 1` stubs. `FRTWDeck*` adds card, remove card, focus, spellbook colour and holocron attach. *client* The collection side is `CollectionData` (51), `CollectionItem` (52), `InstanceID`, and the `SynchronizationCommand_*` family (Start, StartInstances, SendInstances, Complete, StartDelivery, ProcessFulfillment, RemoveInstances, SetDesired, AddNonTradable ..., 21 classes). Trading uses `TradeCommand_*` (25 classes) and posted trades (`RequestPostedTrades` 401, `AcceptPostedTrade` 397, `CancelPostedTrade` 399). Holocron (gem) commands: `RequestAttachHolocron` (571), `RespondAttachHolocron` (572), `RequestConsumeHolocronCharge` (573), `RespondConsumeHolocronCharge` (574). Reading "holocron" as the Treasure War "gem" is inference from `Tcg_imbueHolocrons`, the `TCCW_ImbueHolocronsDialog` movie, and the video's "View Gems".

**Tournaments and events.** 28 `Tournament*` UI handler classes (`tournamenthandlers.cc`) plus `TournamentServiceBase`: join and leave lobbies and sub-lobbies, select tournament deck, show details, scores and rewards, event leaderboards, sessions for a day, join and cancel the matching queue, query event tickets, ready status, timed deck selection, duelist stats. Reward dialogs are unfinished in this build (strings "TODO: Reward Info Dialog" and "TODO: Detailed Rewards Popup Information Here"). The leaderboard row serialiser (`013e52d0`) writes Position, CharacterName, ObjectiveScore, TotalGames, TotalWins, TotalTies, Rating and an int-to-int map; a leaderboard wraps up to 10 rows and an event day writes Year, Month, Day. *client*

**Store, claim and delivery.** `FRTWOpenStore` (`012608e0`) opens the marketplace with StoreLocation, CategoryID and BundleStr; `ECommCommand_RedeemItemGrant` (441, v3, handler queues event `0xa3`) is how the card server grants an item; `AccountCommand_RedeemCode` (432), `ECommCommand_OrderHistory` (440) and the Delivery window carry transaction ids. A context-menu builder (`TcgContextMenu_Build` `013be8e0`) offers add or remove friend, add or remove ignore, chat and profile for a target account. *client*

### Data and formats

**Client resources and folders (read by the client, not sent).**

| File | Contents | Source |
|---|---|---|
| `Resources/MiniGameData.txt`, `MiniGameTypeData.txt` | Caret-separated, `#` header row `ID^TYPE^NAME_ID^DESCRIPTION_ID^ICON_ID^DIFFICULTY^MEMBERS_ONLY^SHORTCUT^MIN_PLAYERS^MAX_PLAYERS^START_SOUND_DEFINITION^PRELOAD_FILE^LEADERBOARD_ID^`. 783 rows; TYPE 3 x4, TYPE 16 x124, TYPE 23 x1. The type table marks types 3, 16 and 23 `IS_FLASH=1`, but the code path is native (see Launching). `MiniGameGroupLinks.txt` puts 68 of the type 16 rows in group 16, the four TYPE 3 rows in group 3 and the Treasure War row in group 23. | client file |
| `Resources/TradingCardAssets.txt`, `SimpleCardAssets.txt` | Lists of `.lst` asset sets (`TCCW_*`, `TC_*`, tooltips, `tieEffect_*`), card thumbnails, and for Simple Card the card-destroyer animations (Ninja, Robgoblin, Yeti) and `campaignImage_101` to `804`. Read with `TradingCardAssets.txt` by `00ad4f50` and `SimpleCardAssets.txt` by `00ad2380`. | client file + pseudocode |
| `Assets_manifest.txt` TCCW lines (84 lines: movies `TCCW_*.gfx` and their `.lst.z` lists) | Scaleform movies for the card UI, loaded from `UI/TradingCard/TCCW_Main.swf`. | client file |
| `ClientConfig.ini` | `AssetDelivery/TcgServerAddress` patch base URL. | client file |
| `tcg/data/` | `archetypes/` (`index.dat`, `storage.dat`, `ruleIndex.dat`, `ruleStorage.dat`, `actionIndex.dat`, `actionStorage.dat`, `msgIndexen.dat`, `msgStorageen.dat`), `collections/` (`default.cln` and three numbered `.cln`), `decks/tournament/` (empty nested folders only), `tutorial/` (`index`, `objects`), `campaign.dat` (668 bytes), `ban.dat` and `restrict.dat` (4 bytes each). The manifest ships most as zlib `.z` files with CRC and size. | client files |
| `tcg/locale/en_us_data.dat` and `.dir` | The TCG's own locale: 5,092 entries (tags `ucdt` x5,013, `ugdt` x79) in `id TAB tag TAB text`; the `.dir` header names the source game "LON", version 2.0.760740, 2013-03-19. Card text, starter-deck names and rules text live here; the main client locale has no `Tcg_` keys. | client files |
| `tcg/data/campaign.dat` | One campaign "Free Realms: The Adventure Begins!" (thumb `campaign_thumb_part1`), a prompt to choose a sphere, "Scenario 1: Fight the Ninja" (Kai) with action `ExecDemoScenario` and the 12-point sentence. | client file |

**In-exe stores** (formats from the code, contents not decoded in this pass). `ArchetypeDB` (`012b8530` index, `012b8ff0` get, `012b87e0` decode): one `DirDBM` per archetype under `./data/archetypes`, each with an `index` record (16-byte entries) and a `storage` record that is a zlib 1.2.3 blob (`01539dc0`); decode is uncompress, factory lookup by class id, dynamic cast to `Archetype`, `Read`, register. Ids at or above 101,000,000 are not stored but built as `CWArchetype` (class `0x222f4`); ids offset by 300,000 redirect to the store; the default archetype is 2547 (`0x9f3`). `ActionDB` (`actionIndex` with four-int records and `actionStorage`), `MessageDB` (per language) and `DeckDB` (`data/decks/*.ldc`) follow the same pattern; `CollectionDB` reads `data/collections`. The `odb` object database (`/objects`, `/index`, `/.clean`) XORs bytes with `0x46`. `NetworkCommand_UpdateDataVersion` (546, v2: a data version and two string lists) drives `CacheDataVersion` (`01250230`), which closes the client if a cache check fails ("SIMPLE TCG ERROR: Failed to cache data! Client Version: %d Incoming Data Version: %d"). The baseline client has **no** `data/` folder beside the exe; only `tcg/data/` exists. *client*

**Card attributes.** `CardAttributeNameForId` (`0131cee0`) maps about 100 attribute ids to keys: 1 Title, `0x1c` CardType, `0x276` GameText, `0x283` Lore, `0x2fa` ArchetypeId, `0x229e9` Charges, `0x229f8` IsPromo, `0x1b327` to `0x1b480` cost names; Traits, Value and Color are `0x22972` to `0x22974`. `0131e290` serialises a card as `<Element>` with ElementID, ArchetypeID, PlayerID, MyElement, Title, Traits, Color, Value, GameText, Illustrator, FlavorText, CollectorsNumber. Card text keys are `Tcg_title_%d`, `Tcg_subtitle_%d`, `Tcg_game_text_%d`, `Tcg_lore_%d` and `Tcg_traits_%d`, with the archetype id as `%d` (above 1e8 the 100000 digit is removed first). Name tables hold the sphere names (Nature, Chaos, Machine, Order), colours, card types (Avatar, Creature, Trick) and subtypes (`01517320`). Treasure War colours are the modifier colours `0x222e6` (red), `0x222e7` (green), `0x222e8` (blue); use points are `0x222e1` to `0x222e3` (thisFight, nextFight, nextDuel, nextBaseAttack, S0660). *client*

**ValueData.** The universal tagged value in UI actions, rules and commands: tag 1 bool, 2 int, 3 string, 5 expression tree, 6 int list, 7 string list, 8 ValueData list, `0xd` int-to-int map, `0xe` int-to-ValueData tree (`01301bc0` reads it). Property keys are integers (for example list key `0xdc` keywords, `0xa2` suppress effects, `0x165`); most are unnamed. Keyword values use bit 31 as a "not" flag. *client*

**Serialisation.** Every network or stored object starts with a class header: `WriteClassHeader` (`013aede0`) writes the class id then the version, and `ReadClassHeader` (`013aee40`) reads the id, checks it against the class's id and reads the version. **Each inheritance level writes its own header**, so a derived command repeats its class id and version once per level (for example `RequestAttachHolocron` 571 v1 three times, a rule-node leaf four times). Integers use `SerialAppendInteger` (`013c23f0`): a compact variable-length form (low nibble, byte count in bits 5 to 7, sign in bit 4) or network-order `htonl`, selected by the global `DAT_01cbdd7c`; who sets that global is unknown. Readers frequently gate fields on the version, while writers often emit the newest layout, so the server should send each class at its **current** version. Objects are created by class id through the registry `013ce130` / `013ce230`. *client, S0723, S0727*

**Command id table** (decimal class ids, from the serialisable-class registry of 520 classes; version in brackets):

| Family | Count | Notable ids |
|---|---|---|
| `GatewayCommand` | 2 | 413 base, 414 `GetConnectionServer` (1) |
| `LoginCommand` | 13 | 78 `GetChallenge`, 79 `SendChallenge` (2), 80 `RespondToChallenge`, 81 `Results` (6), 323 `GuestLogin`, 408 `RegisterUsername`, 410 `SendPassword` (7), 411 `SendSessionID` (7) |
| `NetworkCommand` | 17 | 105 `UpdateLag`, 107 `ErrorMsg` (2), 112 `Ping`, 134 `ConnectionTimedout`, 136 `SystemMessage`, 278 `Disconnect`, 312 `Time` (2), 416 `Gift`, 456 `EULA`, 457 `News`, 458 `LeaderBoardData`, 507 `SessionKick`, 546 `UpdateDataVersion` (2), 575 `RequestData` |
| `AccountCommand` | 26 | 114 `IntroduceAccount`, 118 `SetPreferences`, 415 `SetCampaignStatus` (2), 432 `RedeemCode`, 481 `AddAchievement`, 487 `RecordSinglePlayerGame`, 488 `SetChallengeStatus`, 501 to 503 avatar images, 536 to 541 stadium |
| `LobbyCommand` | 55 | 92 `Join` (5), 93 `Leave` (3), 94 to 96 groups, 97 `SendText`, 106 `BulkJoin`, 137 `ChangeStatus`, 163 `AdjournedGame`, 265 `OfferAdjournedGame`, 266 `AcceptAdjournedGame`, 293 `UpdateTournament`, 298 `Concede`, 299 `SetTournament`, 300 `StartTournamentRound`, 303 to 307 draft, 335 to 347 league and boosters, 412 `UpdateLobbyStats`, 460 `StartTournamentMsg` |
| `MatchCommand` | 12 | 99 `SetReady`, 100 `SetBid`, 101 `SelectDeck` (2), 102 `SelectStartPosition`, 116 `LaunchGame`, 124 `SetMatchState`, 125 `BidPlaced`, 294, 295, 470 `SetDeckArchetype`, 486 `InitiateChallenge` (2) |
| `EventCommand` | 25 | 478 base, 479 `SetReady`, 514 `NextMatchingTime`, 515 to 520 leaderboard, session info and totals, 521 `RequestTimedDeckSelection`, 522 `SendDontPlay`, 524 `ForceIntoEvent` (2), 525 to 528 player and duelist stats, 531 `NotifyMatchEnded`, 532 to 535 current session and sessions on day, 570 `RequestJoinMatchingQueue` |
| `GameCommand` | 43 | 55 base, 56 `IntroduceCard` through 73 `SynchPoint`, 117 `ReadyForStartOfGame`, 128 `RevealCards`, 130 `DiscardCard`, 132 `FatalError` (3), 160 to 165, 262 `SendSerializedGame`, 263 to 267, 308, 314, 320, 322, 325, 326, 383, 471 |
| `CommandObject` | 59 | rule results with do and undo (class ids 178 to 472, see S0786 to S0789) |
| `SynchronizationCommand` | 21 | 82 to 87, 141 `StartDelivery` (6), 142, 143, 271, 301, 302, 313, 316, 475 to 477, 571 to 574 |
| `TradeCommand` / `DeckCommand` / `PollCommand` / `ECommCommand` / `ArchCommand` / `UChatCommand` / `GuildCommand` / `UserDataCommand` | 25 / 4 / 4 / 3 / 7 / 18 / 19 / 26 | trade 146 base; deck 360 to 363 (`AddOnlineDeck`, `DeleteOnlineDeck`, `PopulateOnlineDeckData`); poll 391 to 394; commerce 439 to 441; archetype admin 447 to 452, 469; chat; guilds 418 to 429, 461 to 468 |
| Whole-state classes | 1 each | `CTP::Game` 166 (v52), `CWGame`/`CWMainController` 140003, `GameTurn` 232 (v17), `CWGameTurn` 140004, `Card` 168 (v5), `CWCard` 140001, `PlayElement` 167, `Player` 261 (v4), `PlayArea` 172, `Archetype` 3 (v7), `Action` 18 (v11), `TradingCard::Lobby` 103 (v3), `Deck`, `CollectionData` 51 (v3), `CollectionItem` 52 (v7), `ValueData`, `StateMachine`, ... |

Field-level layouts known from the readers: `LoginCommand_SendSessionID` (v7): SessionID, Platform, Match Challenge Key, Is Parent Session ID, Login Client, Locale, CharacterName, CharacterID, Ticket, an int map (keys 1, 2, 4) and a flag. `LoginCommand_Results` (v6): succeeded byte, explanation int (v3+), AccountID, use-explanation byte (v2+), ServerID (v4+), session string (v5+), restrictions (compressed from v6). `GatewayCommand_GetConnectionServer`: SessionID, Username, Host, Port, Success byte, Error Type; its result queues client event `0x95` (fail) or `0x96` (success). `CollectionData` (v3): ID, name, notes, item map, owner account id, owner name, type (v2+), ModID (v3+). `CollectionItem` (v7): ArchetypeID, Quantity, Foils (written as int, read as bool), Instances vector, NonTradableInstances. `AccountCommand_SetCampaignStatus` (v2): four ints (campaign id, status, difficulty, custom int). `GameCommand` base: PlayerID, GameID, time value (v2+). *client; names are ours where the client shows none*

**Error codes.** `012e1020` turns a numeric code into a locale key; the full table, decoded from its jump table, has 123 codes. Server-visible groups: 1 to 13 login, connection and proxy errors; 14 to 16 game, data and action version mismatch (`invalidGameVersionError`, `invalidDataVersionError`, `invalidActionVersionError`); 17 to 25 registration, entry-fee, tournament and match-join errors; 26 to 30 game and lobby server down; 31 to 33 deck not received or unowned; 34 to 38, 46, 91, 92, 121 trade errors; 39, 41, 42 and 47 league; 44 `outOfSyncError`; 48 to 54, 93, 94 guild; 55 `serverBusyError`; 59 to 86 login (invalid session, banned, secure-id, migration, with 76 to 79 unused) and 119 beta flag; 87 to 90 gateway; 95 to 100 redeem-code; 102 to 105, 116 to 118, 120 and 122 item grant, 101 migration of a banned account; 106 to 115 chat; 123 underage. Codes 40, 56 to 58 and 76 to 79 map to `unknownError`. The strings keep original typos (`gameServieLoginError`). *client*

### Network

The card feature uses **four separate links**. Only the first is the Free Realms game connection.

**1. Free Realms game connection (family 39, mini-game).** Header on every family-39 packet, from the reader `009b2340` and the base constructor `008a4420`: `i16` opcode 39, `i8` sub-op, `i32` state id, `i32` (header slot `+0x14`), `i32` (header slot `+0x10`). The two trailing header ids are written in reverse argument order; the Kart note shows the same header on 39/7 ([[Kart Racing]]). *client*

| Sub | Class | Dir | Payload | Handler / sender | Status |
|---|---|---|---|---|---|
| `0x26` | `RequestTCGChallengePacket` (ctor `009a9ec0`) | C>S | header only; the one id sits in the state slot | `RequestTcgChallenge` `009bdaf0`, no static caller found | confirmed class, likely sender |
| `0x28` | `TradingCardStartGamePacket` (ctor `009a9de0`, reader `009b7f30` / `009b5ec0`) | S>C | header, then `i32 a`, `i32 b` (a 64-bit launch ticket as two halves) | `DispatchMiniGamePacket` `009bf890`: type 23 gets (a, b, third header id) via slot 27; type 16 gets (a, b) via slot 25 | confirmed |
| `0x29` | load failure | S>C | header | runs Lua `TradingCardGameHandler:OnLoadFail` | confirmed |
| `0x25` | `SendMiniGameSubop37` (`009bf590`) | C>S | header | sent by the Treasure War state machine on entering state 5 | likely label |
| `6` | `SendMiniGameLeaveSubop6` (`009be8e0`) | C>S | header | sent when the card client closes | likely label |

Other family-39 sub-ops (`0x12` game over, `0x15` stars earned, ...) belong to the generic mini-game flow ([[Minigame Framework]]); this guide found nothing TCG-specific in them. Packet `PacketGeneratePortraitRequest` (BaseFotomat sub-op 1) and `AccountCommand_RequestAvatarImages` (501) / `SendAvatarImage` (502) / `UploadAvatarImage` (503) move character images. *client*

**2. HTTP asset delivery.** `AssetDelivery/TcgServerAddress` is the base URL for `AssetsTcg_manifest.txt` (one `name,crc32,size` line per file, 59 lines in the baseline) and the pack `AssetsTcgW_000.pack`. A protocol-level reading of `HttpLibrary::HttpConnectionHandlerAdapter` (`vtable 01879cd8`) exists (headers, counted and chunked bodies) but its use for the card assets is not traced. *client*

**3. The in-exe card service connection (Treasure War).** A direct TCP link, not tunneled through the game packet, built by `NetworkService`.
- **Where.** `LoadHostSvrConfig` (`012f1b80`) reads `host.svr` with keys `host`, `port`, `ghost`, `gport`; the baseline ships no such file, and `CardClientInit` fills the same statics from baked values per environment (`DAT_01bb94a0`, `DAT_01bb94bc`, `DAT_01cbd968`, `DAT_01cbd96c`). Live (environment 0): `lvsccg-01-01.station.sony.com` with `0x5517` (21783) and `lvsccg-01-02.station.sony.com` with `0x5516` (21782). Every other environment uses `sdtccg-dev1.station.sony.com` with the port pair 21283/21282 (main), 21583/21582 (qa), 21183/21182 (stage), 21383/21382 (livetest). Which static is the game host and which the gateway is inferred from the key names. *client disassembly*
- **Connect.** `012f1e00` resolves, opens a non-blocking socket and sends `GatewayCommand_GetConnectionServer` (414) **without** the routing triple (`[htonl(body length)][body]`); the reply (SessionID, Username, Host, Port, Success, Error Type) points at the real connection server. Entering the main mode frames `NetworkCommand_RequestData` (575) carrying `0x9f3`. Logout sends `NetworkCommand_Disconnect` (278, string "logout"). The client can also tunnel this through a SOCKS4 or SOCKS5 proxy (`012f1180`, `012f1400`); a revival server does not need to implement that. *client*
- **Framing.** Outbound (`012f0a00`): `[htonl(body length + 12)][three htonl routing dwords][body]`. Inbound (`012f2150`): `[htonl(N)][class id][body]` with no routing triple; the object is created from the class id, cast to `NetworkCommand` and read. The body is the class-header form described under Data and formats.
- **Routing dwords** (virtual slot 21): `1000000000` default for Lobby, Account, Login, Network, Match, Synchronization, Arch and UChat commands; `1000000001` for `GameCommand` with the game id as the third word (`0141afd0`); `1000000002` for trade with the trade session as the third word (`014426a0`); `1000000003` guild (`014d67a0`); `1000000004` commerce (`014e3590`); `1000000005` event with the account id as the second word (`014473e0`); `1000000007` user data with the account id second (`01528a80`). `CommandObject` has no routing (it is carried inside a game command). One trade class overrides to the default (`014a8560`). *client*
- **Login.** The Free Realms flow is `LoginCommand_SendSessionID` carrying the game session id and launch ticket; the reply is `LoginCommand_Results`. The password path exists too: `SendChallenge` carries a challenge and a "use MD5" flag, and the client answers with a 16-byte MD5 or a 20-byte SHA-1 as 40 hex digits (`014b5da0`); the digest inputs are not proven. The standard hash code is at `01519900` (SHA-1) and `0151a6e0` (MD5). *client*
- **Keep-alive and system.** `Ping` (112), `UpdateLag` (105), `Time` (312), `SystemMessage` (136), `SessionKick` (507), `ConnectionTimedout` (134), `QueryDisconnect` (133), `ErrorMsg` (107, carries an error code from v2). *client*
- **What the server pushes.** Lobby snapshots (103), group and buddy lists, tournament and league state, draft state, match state and bids (`LaunchGame`, `SetMatchState`), game commands and the whole-game snapshot, collection and delivery synchronisation, trade and posted-trade state, account introductions and avatar images, event leaderboards and session info, polls, MOTD (`DisplayMOTD` 309, `UpdateLobbyStats`), news, EULA, achievements, and item grants (`RedeemItemGrant`). The client's replies are listed in the command table above. Handler bodies that are `mov al,1; ret` stubs fit client-to-server requests (inference, S0778). *client*

**4. The DLL's own link.** Unknown. The DLL receives `--sessionID`, `--ticket`, `--character`, `--challenge`, `--username`, `--realm` and `--host=fr`, and `OnMMODisconnect` / `OnMMOSystemMessage` are exported for the exe to report game-connection events to it. Whether it speaks the command protocol above is **not known**.

### What the Rust server must do

Items marked (R) are reconstructed policy, not original behaviour. Items marked **blocked** need the DLL analysed first.

**A. Entry and launch (both games)**
1. Serve the mini-game rows `MiniGameData` TYPE 16 (124 rows plus lobby 499, tutorial 537, Kai 1036) and TYPE 23 (727) and answer the activity-join request from `StartTutorial` / `StartTreasureWar` by creating a mini-game state of the right type. Game identity is the state type: 16 for the DLL game, 23 for Treasure War.
2. Mint a **64-bit launch ticket** per launch and send `39/0x28` with it as two `i32` halves in the payload. For type 23 also fill the header's `+0x14` id slot with a positive campaign mission id to open that mission directly, or zero for the home screen (the mission-id reading is likely, not proven). Send it **only after** the client has reached state 5 (it logs and ignores a ticket in any other state; the client sends `39/0x25` on entering it) (R: wait for `0x25`, allow the client's own timeout path).
3. Send `39/0x29` when the card service cannot be reached so the client shows the portal and the server-error string.
4. Accept `39/6` (leave) and `39/0x25`, close the mini-game state, and restore the player to the world; accept `39/0x26` (challenge) once its trigger is identified.
5. Store-reply and claim-reply packets must carry `tcgItemEquipped` / `twItemEquipped` so the Lua launches the matching game after a pack or deck item is equipped ([[Coin Store and Purchases]]). Hide the TCG category 83 and refuse type 16 launches for Apple clients (R, matching the client's own refusal).

**B. Treasure War card service (a separate server)**
6. Listen where the card client connects: the baked `lvsccg` / `sdtccg-dev1` names and the port pairs above, or a patched `host.svr` / environment table (R). Implement the gateway step (`GetConnectionServer` 414 with the unrouted `[len][body]` form) and the framed command stream (`[len][class id][body]` in, `[len+12][3 dwords][body]` out).
7. Authenticate with `LoginCommand_SendSessionID` against the game's session and the launch ticket, reply with `LoginCommand_Results` (set the version fields the reader needs: v6 or later), then introduce accounts (`AccountCommand_IntroduceAccount`) and push the lobby snapshot (103 v3) with the five role sets. Map failures to the 123 error codes (use `invalidDataVersionError` 15 for stale data).
8. Serve card data: archetypes, rules, actions and messages as the client stores expect (`data/archetypes` family, current class versions), announce the data version with `UpdateDataVersion` (546), and make sure every id in a deck, collection and rule resolves. **How the exe obtains this data is unresolved** (open question 3).
9. Run matches: `LaunchGame` (116) to create the client game; `SetupGame` (67 v10) with ordered accounts, versions and match structure; wait for `ReadyForStartOfGame` (117) from every client; then own turn order, legality and rule evaluation. The client contains the full rules engine, so the server can either host the same rules (the shared `common\rules` code suggests this) or relay commands and accept the clients' results; which the original did is unknown (R: server-authoritative, rebuild rules from data).
10. Send commands to the right game with routing word `1000000001` and the game id; accept `CardPlayed`, `ActionPlayed`, `ButtonPressed`, `StateSpecificMessage`, `SynchPoint`, `DiscardCard`, `ConcedeGame`, `OfferDraw`/`AcceptDraw`, `OfferAdjournment`/`AcceptAdjournment`, `PrePass`, `OutOfSync`, `FatalError`; drive the client's state machine by naming the next state id; implement the timeout path (client submits `RemovePlayer` reason 2 at the limit) and `SendSerializedGame` for resync.
11. Persist results: `RecordSinglePlayerGame` (487) and `SetChallengeStatus` (488) for AI games; match statistics (result, turns, cards played, cards destroyed); campaign progress with `SetCampaignStatus` (415) and the eight-campaign screen data (purchased, cost, boss unlocks).
12. Collections and decks: `Synchronization*` start, instances, delivery (transaction id, source, silent flag), fulfilment and non-tradable flags; `CollectionData` v3 and `CollectionItem` v7 with archetype, quantity, foils, instance lists; deck save, load, delete and validation messages; gem (holocron) attach, charge consume and the matching `Respond*` commands.
13. Trading: trade session routing (`1000000002`), posted trades (request, accept, cancel with server confirmation: the client drops an entry only on the cancel handler), limits and the trade error codes.
14. Events and tournaments: event session list per day, matching queue, leaderboard rows (Position, CharacterName, ObjectiveScore, TotalGames, TotalWins, TotalTies, Rating), player and duelist stats, deck selection with a timer, ready status, and the organized-play scoring in the client text (3 points per win, 1 per loss, tie-breaker from daily wins, 14-day ranking) (R for everything beyond that text).
15. Commerce: grant pack contents and virtual reward cards through `RedeemItemGrant` (441 v3) and `RedeemCode` (432); pack contents of 10 cards plus 1 virtual reward are [wiki] and [client locale], odds are not recovered (R). Keep trade and redeem limits server-side.

**C. TCG (DLL) service** **(blocked)**
16. Analyse `FreeRealmsTCG.dll` (exports, network client, data readers) before promising server behaviour. Until then the server can only: create type 16 states, mint and accept the launch ticket and session data it passes on the command line, and serve `AssetsTcg_manifest.txt` and the pack at the `TcgServerAddress` URL.
17. Serve the TCG's content as data that the original shipped: the 124 duel rows with 12-point rule text and prize cards, the card text locale (5,092 entries), sets and starter decks, and the Card Duelist content in [[Card Duelist]].

**D. For the new client (design inputs, R)**
18. Treat the rules as data: the shipped rule stores, node classes and command objects are a complete description of one engine; a new client can reuse the data model without the Scaleform or Qt layers.
19. Keep Treasure War and the TCG as separate games with separate lobbies, collections and event programs, as the original does (separate boosters, separate organized-play rules text, separate store categories).

### Evidence

- **Client functions.** 3,412 accepted functions across 86 shards: 1,463 confirmed, 903 likely, 722 speculative, 324 unknown; only 2 carry a `wire` link (`009a9de0`, `009a9ec0`) because the packet catalogue has no card-game rows; the 520 serialisable command classes come from the map's command registry and `nmq command`. Subsystems: `minigame.tcg` 2,160, `ui.tcg` 321, `net.tcg` 225, `ui.deck` 143, `minigame.tcg.query` 134, `.data` 120, `.rules` 102, `.command` 64, `.action` 42, `.property` 34, `.player` 33, `.game` 25, `.playelement` 9. The Quests guide's note that 10 of the 17 `game.tutorial` rows are card-game code is confirmed: the 10 are `TutorialDB` and its helpers (`01405d60`, `01406020`, `01406350`, `014064e0`, `01406950`, `01406cc0`, `01406d50`, `01406dd0`, `01406df0`, `01406ec0`), the card rule-set store; the other 7 are `ClientTutorialManager` (4) and the three `+0x780` helpers (`008a89a0`, `008a8a30`, `008a9200`) of the new-player tutorial. A great deal of surrounding code (STL, ValueData accessors, command framing, login and system commands) is mapped under other subsystems and is described here from the same shards.
- **Shipped files read this pass.** `Resources/MiniGameData.txt`, `MiniGameTypeData.txt`, `MiniGameGroupData.txt`, `MiniGameGroupLinks.txt`, `TradingCardAssets.txt`, `SimpleCardAssets.txt`, `ClientConfig.ini` (the `Tcg` key only), `Assets_manifest.txt` (TCCW lines), `tcg/AssetsTcg_manifest.txt`, `tcg/data/campaign.dat`, `tcg/locale/en_us_data.dat` and its `.dir` header, directory listings of `tcg/`, `tcg/data/`. **Not opened:** `tcg/frtcg.ini`.
- **Client scripts and locale.** The decoded `ScriptsBase.lua` (launch paths, handler, store flags), the locale text for packs, organized-play rules and mini-game names, and `lab/reports/locale-id-to-text.json` for the mini-game name and description ids.
- **Video.** 99 reviewed frame records across 45 videos show the feature; 27 frames across 12 videos are Treasure War. Used: lobby, welcome panel, collection, redemption and delivery, deck builder, result stats, event calendar, advertisements for Sets 3, 4 and 5, Treasure War duel screens. No chat or player names are recorded; frames with `[player]` placeholders are described without names.
- **Wiki.** The Treasure War and Trading Card Game pages (rules, gems, booster contents, series order); the Card Duelist job page in this vault carries the NPC and quest detail.
- **Vault.** Piece 010 (mini-game table counts and locale ranges re-derived from the client files here), Piece 019 (event scoring text), Piece 015 (type table), the Quests and Objectives guide (the `game.tutorial` note). Their community-catalogue parts were not used.

### Open questions and contradictions

1. **The DLL.** `FreeRealmsTCG.dll` has not been analysed: its network protocol, server, data readers, NPC-duel AI, rules and statistics are unknown. This is the largest gap in the whole guide.
2. **Which game does the mapped engine serve?** The evidence for Treasure War is strong (type 23 routes to it, `TW` campaign titles, `FRTW*` handlers, video and wiki), but the exe also contains the Clone Wars engine ancestry (window title, thumbnails, a record "Cham Syndulla / Question the Captain" in a second campaign-like table, `01328400`). Whether any TCG (type 16) behaviour runs in the exe is unproven.
3. **Where Treasure War's card data comes from.** The exe reads `./data/archetypes` and neighbours, but the baseline client has no `data/` folder beside the exe. Candidates: the server pushes it after `UpdateDataVersion` (`CacheDataVersion` only checks caches), the exe resolves the folder elsewhere, or it uses `tcg/data`. Unresolved.
4. **Wire form of integers.** Compact variable-length or `htonl` depends on `DAT_01cbdd7c`; the setter is unknown. Command header reads depend on the version register in several `Read` functions that Ghidra mislabels (S0731, S0738, S0774).
5. **Which port is which.** Host and gateway roles of the two `lvsccg` statics and their ports are inferred.
6. **Mission id in `39/0x28`.** The third value is read as a campaign mission id from states 3 and 4 only.
7. **`RequestTCGChallenge` trigger** and the Lua `Shutdown` entry (`00c04920`, unmapped).
8. ~~**`TradingCardGame.StartTutorial` naming.**~~ Resolved (T504): `StartTutorial` (`00c04970`) joins activity `Tcg.ActivityId` (key `0x6d54a188`, default 1049, MiniGameData 537, the tutorial row) and `StartTreasureWar` (`00c049f0`) joins `Scg.ActivityId` (key `0xbfd77cb0`, default 1078, MiniGameData 727); both are 167/1/2 through `00be7d80`.
9. **TYPE 3 rows.** Rows 7, 11, 107 and 108 are TYPE 3; only row 11 has TCG text ("FR Trading Card Game / FIGHT!"). Piece 010 found rows 7 and 108 resolve to unrelated text and 107 to nothing. What TYPE 3 launches is unknown. Mini-game group 57 (`gotoPortal`) is not a row in `MiniGameGroupData.txt`.
10. **Campaign table.** 74 mapped entries versus 40 test missions (`BuildTestCampaigns`) versus 8 campaigns in the screen builder; the `Tcg_Opponent` strings are not in the main client locale, and the shipped `campaign.dat` has a single scenario.
11. **Deck builder in Treasure War.** The wiki says decks are drawn at random from owned cards; the client has `FRTWDeck*` add and remove actions and a spellbook colour setter. Which is live is unknown.
12. **Rules sharing.** Whether rule evaluation ran on the client, the server or both is open (S0717). Server-authority is suggested by the "Server ignoring game commands" flag and the whole-game snapshot, but not shown.
13. **Original-client inconsistencies to avoid copying:** `Deck` text save writes `Card N: a.b (count)` but load parses `(ID,Instance)` so a round trip likely misparses (S0730); `CommandObject_DiscardPile` writes five values but reads four; `DeckCommand` write and read disagree on the vector element type; `ActionPlayed` debug print uses target 2 twice; `CTP::Game` repeat check logs "more than 3 times" but tests `> 5`; `DivideExpression` traces "PREVENT DIVIDE BY ZERO" but divides anyway; `Each::Evaluate` always returns true.
14. **Rewards.** Card rewards are named in the client text; coin and XP amounts per win, pack odds and Treasure War prize tables are not recovered. The reward dialogs are `TODO` strings.
15. **Names.** Row 727 reads "Treasure War" but the store listing calls its category "Treasure War" and the boosters "Treasure War Booster Pack"; the TCG appears as "Free Realms Trading Card Game", "FR Trading Card Game", "TCG" and "Trading Card Game" in different places. The shard labels use `Tcg` for the engine of both.
16. **Unlabelled but central:** `00ad47f0` (DLL startup), `010d1ab0` (module load), `012f2150` (inbound framing), `012e1020` (error map), `0131e290`, `01329630` (campaign table), `0136ea70` (flush) carry no accepted label; their roles here come from pseudocode read in this pass or the shard notes.

### Key functions

| Address | Role |
|---|---|
| `009bf890` | `DispatchMiniGamePacket`: type 23 to the Simple Card manager, type 16 to the DLL manager; `0x28`, `0x29` (confirmed) |
| `009a9de0` / `009a9ec0` | `TradingCardStartGamePacket` / `RequestTCGChallengePacket` constructors, sub-ops `0x28` / `0x26` (confirmed) |
| `009bdaf0` | `RequestTcgChallenge` sender (likely) |
| `00c04950` | `RegisterTradingCardGameBindings`, the Lua `TradingCardGame` module (confirmed) |
| `00ad2d30` / `00f9d320` | `InitCardClient` and `CardClientInit`: boot the in-exe card client, realm and environment (confirmed / likely) |
| `00ad2380` | Treasure War manager state machine, `SimpleCardAssets.txt`, `goCampaign` and friends (confirmed) |
| `00ad1a80` | `HandleTcgLaunchTicket`: accepts the ticket in state 5 only (confirmed) |
| `00ad4f50` / `00ad3670` | DLL manager update and asset start (confirmed) |
| `00ad47f0` | DLL manager `Startup`: argument builder and state 3 (no accepted label) |
| `010d1ab0` / `010d25f0` | DLL load with 21 exports / manager singleton (no label / likely) |
| `010ce4f0` | `GuiTradingCardCtrl` key mapping for the DLL window (no accepted label) |
| `012f1b80` / `012f0a00` / `012d0810` | `host.svr` loader, command framing, send (likely) |
| `013aede0` / `013aee40` | Class header write and read (confirmed) |
| `0141afd0` | `GameCommand` routing triple `1000000001` (confirmed) |
| `0124f780` | `LoginResultsDisplayAction::ApplyLoginResult` (no accepted label) |
| `01250230` | `CacheDataVersion`: data version check that closes the client (confirmed) |
| `01336d20` | `CWGame::Setup` (confirmed) |
| `01337510` | `CWGame::DispatchTextCommand` (confirmed) |
| `0136d640` | `DispatchCommandObject`, with the "Server ignoring game commands" guard (likely) |
| `01363270` | `CTP::Game::SetGameWonByPlayer`, client-side win (confirmed) |
| `013734b0` | `CTP::Game::Concede` (confirmed) |
| `01331120` / `01332ed0` | `MatchWonByGameCount` / `DeliverGameEndNotice` (confirmed) |
| `01373e40` | `CTP::Game::Write`, the whole-match serialiser (confirmed) |
| `01314350` | `CWMainController` ctor (confirmed) |
| `014050d0` | `CWAIController::handleScript` (confirmed) |
| `012b8ff0` / `012b8530` / `012bd4a0` | `ArchetypeDB` get / index load, `ActionDB` load (confirmed / likely / confirmed) |
| `01539dc0` | zlib 1.2.3 uncompress used by the stores and commands (confirmed) |
| `01301bc0` | `ValueData::Read` (confirmed) |
| `0131cee0` / `01329630` | card attribute id to key / campaign table of 74 entries (confirmed / unlabelled) |
| `013ed580` / `013eb7b0` | deck format validation / minimum draw-deck rule (confirmed / likely) |
| `013aa220` | deck validation messages (confirmed) |
| `01406d50` / `01406ec0` | `TutorialDB` / `LoadRuleSet` (confirmed) |
| `01419a30` / `01419f70` | MT19937 init and bounded draw (likely / confirmed) |
| `013e52d0` | leaderboard row XML (confirmed) |
| `00b4e9f0` | `CaptureTCGPortraitToBuffer` (confirmed) |
| `012e1020` | numeric error code to `Tcg_error.*` key, 123 codes (no accepted label) |
<!-- NARRATIVE END -->

## Mapped functions (generated)

3412 accepted functions: 1463 confirmed, 903 likely, 722 speculative, 324 unknown. Network-linked (`wire`): 2.

Subsystems: `minigame.tcg` (2160), `ui.tcg` (321), `net.tcg` (225), `ui.deck` (143), `minigame.tcg.query` (134), `minigame.tcg.data` (120), `minigame.tcg.rules` (102), `minigame.tcg.command` (64), `minigame.tcg.action` (42), `minigame.tcg.property` (34), `minigame.tcg.player` (33), `minigame.tcg.game` (25), `minigame.tcg.playelement` (9)

_Large or library guide: only confirmed and likely functions are listed (2366 of 3412)._

### `minigame.tcg`

**Action** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013257f0` | Action::ExpandPresentationText | confirmed | Virtual slot [14] of Action (thiscall; output string, target object). Returns an empty string when the presentation object at this+0x1c is null. Otherwise it r… |

**ActionPlayNode** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0149d9a0` | ActionPlayNode::ActionPlayNode | confirmed | Constructor body: runs the ExpressionTreeNode constructor 0149ab30, stores the ActionPlayNode vtable (018fa014), zeroes the child pointer at +0x14 and the int … |
| `0149da20` | ActionPlayNode::GetTypeName | confirmed | Virtual slot [11] of ActionPlayNode. Returns the literal 'ActionPlayNode'. |
| `0149dba0` | ActionPlayNode::Evaluate | confirmed | Virtual slot [14] of ActionPlayNode. Traces '(Play action container' and switches on the pile at this+0x10: 1 (pile == Target1) and 2 (Target2, with the env cu… |
| `0149dd30` | ActionPlayNode::GetValidTargets | confirmed | Virtual slot [15] of ActionPlayNode (also CWActionPlayNode [15]). Traces '(gvt:ActionPlayNode' and ') => false', then returns 0 with no candidates. |
| `0149deb0` | ActionPlayNode::ScalarDeletingDtor | confirmed | Virtual slot [0]: calls the destructor body 0149da60, then frees the object with _free when bit 0 of the argument is set. Returns this. |
| `0149da60` | ActionPlayNode::~ActionPlayNode | likely | Destructor body: stores the ActionPlayNode vtable; if the child at +0x14 is non-null it calls that child's slot 0 with flag 1 and clears it; then runs the Expr… |
| `0149de30` | ActionPlayNode::Create | likely | Factory: operator new 00d833a6 with size 0x1c; on null it returns 0, otherwise it constructs with 0149d9a0 and returns the object. |
| `0149de90` | ActionPlayNode::RegisterFactory | likely | Registers factory 0149de30 under classID 0x31 through 013ce130 and 013ce230. |

**AddExpression** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0149d630` | AddExpression::AddExpression | confirmed | Constructor body: runs the Expression base constructor 0149adc0 and stores the AddExpression vtable (018f9f6c). |
| `0149d6b0` | AddExpression::GetTypeName | confirmed | Virtual slot [11] of AddExpression. Returns the literal 'AddExpression'. |
| `0149d740` | AddExpression::AssertFalseStub | confirmed | Virtual slot [15] of AddExpression. Calls the assert helper 012f5a60 with 'false', source AddExpression.cpp and line 0x12, then returns 0. |
| `0149d940` | AddExpression::ScalarDeletingDtor | confirmed | Virtual slot [0]: stores the AddExpression vtable, calls the base destructor 0149ae80, frees the object with _free when bit 0 of the argument is set. Returns t… |
| `0149d8c0` | AddExpression::Create | likely | Factory: operator new 00d833a6 with size 0x24; on null it returns 0, otherwise it constructs with 0149d630 and returns the object. |
| `0149d920` | AddExpression::RegisterFactory | likely | Registers factory 0149d8c0 under classID 0x24 through 013ce130 and 013ce230. |

**AndExpression** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0149d340` | AndExpression::AndExpression | confirmed | Constructor body: runs the Expression base constructor 0149adc0 and stores the AndExpression vtable (018f9ec4). |
| `0149d3c0` | AndExpression::GetTypeName | confirmed | Virtual slot [11] of AndExpression. Returns the literal 'AndExpression'. |
| `0149d450` | AndExpression::GetValidTargets | confirmed | Virtual slot [15] of AndExpression. Traces '(gvt:And '. Calls the lhs child (+0x1c) slot [15] with the output and trace; when it returns 1 traces 'First branch… |
| `0149d4e0` | AndExpression::Evaluate | confirmed | Virtual slot [14] of AndExpression. Traces '(And '. Evaluates lhs (+0x1c) through slot [14]; only when it is true evaluates rhs (+0x20) through slot [14]. Trac… |
| `0149d5d0` | AndExpression::ScalarDeletingDtor | confirmed | Virtual slot [0]: stores the AndExpression vtable, calls the base destructor 0149ae80, frees the object with _free when bit 0 of the argument is set. Returns t… |
| `0149d550` | AndExpression::Create | likely | Factory: operator new 00d833a6 with size 0x24; on null it returns 0, otherwise it constructs with 0149d340 and returns the object. |
| `0149d5b0` | AndExpression::RegisterFactory | likely | Registers factory 0149d550 under classID 0x1a through 013ce130 and 013ce230. |

**CTP::Game** (43)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013600a0` | CTP::Game::OfferDraw | confirmed | Virtual slot [120] of CTP::Game (thiscall, one stack argument = offering player id, returns void): clears the flag byte at this+0x361 and traces Offering Draw!… |
| `013602c0` | CTP::Game::OfferAdjourn | confirmed | Virtual slot [28] of CTP::Game (thiscall, one stack argument = offering player id, returns void): when 01353b80 on the argument reports false it clears the fla… |
| `01360530` | CTP::Game::ShowAdjournOfferPrompt | confirmed | Virtual slot [29] of CTP::Game (thiscall, two stack arguments, returns void): when the flag byte at this+0x1d8 is clear, clears the flag byte at this+0x360, tr… |
| `01363270` | CTP::Game::SetGameWonByPlayer | confirmed | Virtual slot 43 of CTP::Game. When the byte at this+0x1d8 is clear: traces 'GAME WON BY PLAYER %d' with the argument, sets the byte at this+0x131, computes the… |
| `01363b70` | CTP::Game::ExpireCommands | confirmed | Virtual slot 88 of CTP::Game. Traces 'Game::expireCommands %d' with the argument, looks up that key in the command map at this+0x108 (01354a00) and walks its e… |
| `01364200` | CTP::Game::ExpireCommandTargetingCardWithDuration | confirmed | Virtual slot 7 of CTP::Game. Traces the call when the card argument is non-null, looks up the duration key in the command map at this+0x108, and walks its (car… |
| `01364df0` | CTP::Game::RemoveCommandsFromCard | confirmed | Virtual slot 90 of CTP::Game. Traces 'Game::removeCommandsFromCard %d (%p)' when the card argument is non-null. Walks the command-duration map at this+0x10c: a… |
| `01369c70` | CTP::Game::DispatchTextCommand | confirmed | Thiscall on a text command line (param_2). Returns at once when the line is null or starts with '#'. Otherwise it copies the line into a std::string, runs gate… |
| `0136ca90` | CTP::Game::SubmitCardCommand | confirmed | Asserts that param_3 (the card) is non-null (FUN_012f5a60 with Game.cpp line 0x1ce7) and that its player id via virtual slot 0x28 is non-zero (line 0x1ce8). It… |
| `0136d1f0` | CTP::Game::AdvanceTurn | confirmed | Logs "inAdvanceTurn". If the setup flag byte at this+0x17c is clear it sets this+0x17d, logs "GameIsSetup = false" and returns 0. If the started flag at this+0… |
| `0136ed60` | CTP::Game::SetupDeckForPlayer | confirmed | Asserts that the player (line 0xafc) and deck (line 0xafd) arguments are non-null. Logs 'setupDeckForPlayer\t%d\t%s\t%s' with the player id and the deck names … |
| `0136f2f0` | CTP::Game::Stop | confirmed | Logs 'STOP CALLED ON THE GAME' (013508b0) and 'stop\t' (013541f0), then calls this->vtable[0](1). Slot 0 of the game's vtable is the scalar deleting destructor… |
| `0136fa30` | CTP::Game::DragStopped | confirmed | Asserts mCurrentTurn (line 0xd78). Logs 'dragStopped\t%d\t%d' (013541f0) and also writes the FormatTimestampedLog copy (0136b570) when +0x14d is clear. Stores … |
| `013700b0` | CTP::Game::CardIntroduced | confirmed | Asserts mCurrentTurn (line 0xe06). Logs 'cardIntroduced\t%d\t%d' (013541f0), stores the clock seconds in the per-key map at param_2 (0051ff49), and sends event… |
| `013706e0` | CTP::Game::ActionPlayed | confirmed | Does nothing unless param_4 (actionID) is non-zero. Asserts mCurrentTurn (line 0xe8f). Logs 'actionPlayed\t%d\t%d\t%d'. When the game is ended (+0x1d8 set) it … |
| `01370af0` | CTP::Game::CardSelected | confirmed | Asserts mCurrentTurn (line 0xf6c). Logs 'cardSelected\t%d\t%d' (plus 0136b570 when +0x14d is clear). When the card argument is 0 it warns 'Game::cardSelected g… |
| `01371070` | CTP::Game::PlayAreaSelected | confirmed | Asserts mCurrentTurn (line 0xff8). Logs 'playAreaSelected\t%d\t%d' (013541f0), stores the clock seconds in the per-key map at param_2 (0051ff49), and sends eve… |
| `01371320` | CTP::Game::TokensMoved | confirmed | Asserts mCurrentTurn (line 0x1032). Logs 'tokensMoved\t%d\t%d\t%d' (013541f0), stores the clock seconds in the per-key map at param_2 (0051ff49), and sends eve… |
| `013715e0` | CTP::Game::SynchPointReached | confirmed | Asserts mCurrentTurn (line 0x106d). Logs 'synchPointReached\t%d' (013541f0), stores the clock seconds in the per-key map at param_2 (0051ff49), and sends event… |
| `013724a0` | CTP::Game::ButtonPressedWithTextAndRevealID | confirmed | Asserts mCurrentTurn (line 0x13a4) and the state machine (line 0x13a5). Logs 'buttonPressedWithTextAndRevealID\t%d\t%d\t%s\t%d'. Calls 013efcc0 for the current… |
| `013727e0` | CTP::Game::RemovePlayerFromGame | confirmed | Looks up the player for account id param_2 with 013504e0. If none is found it logs 'removePlayerFromGame:: couldnt' get Player for accountid: %d'. Otherwise it… |
| `01372a40` | CTP::Game::AcceptDraw | confirmed | Logs 'Accept Draw! %d, %d'. Builds a 0x28-byte GameCommand_AcceptDraw (ctor 01430850, vtable 018e88c4) with player (0141b000) and the accept flag (014309e0) an… |
| `01372ea0` | CTP::Game::AcceptAdjourn | confirmed | Walks the player vector at +0x334..+0x338. For each element it reads the player id (01344e20) and looks it up in the accept tree at +0x1e0 (0129ad50); an id mi… |
| `013734b0` | CTP::Game::Concede | confirmed | Calls 01353b80(player); when it returns 0 it logs 'Player %d has conceded!' and 'concede\t%d', builds a 0x28-byte GameCommand_ConcedeGame (ctor 01431210, vftab… |
| `01373560` | CTP::Game::ConcedeSubGame | confirmed | Calls 01353b80(player); when it returns 0 it logs 'Player %d has conceded sub game %d!' and 'concedeSubGame\t%d', builds a 0x28-byte GameCommand_ConcedeGame (c… |
| `01373830` | CTP::Game::StateSpecificMessage | confirmed | For each non-null pointer argument it copies the text returned by 012fe040 into a local std::string, then logs 'stateSpecificMessage\t%d\t%s\t%s'. Stores the c… |
| `01373e40` | CTP::Game::Write | confirmed | Serializable Write in the secondary CTP::Game interface (vftable 018dcd70, slot 10). With no writer argument it creates one through slot 1; with one it calls s… |
| `01378020` | CTP::Game::DtorBody | confirmed | Thiscall destructor body. Stores the CTP::Game vtables, logs KILLING GAME VIA DESTRUCTOR through 013508b0, unregisters via 0135a400(this,1), clears DAT_01cbdbb… |
| `01378f50` | CTP::Game::Ctor | confirmed | Thiscall base constructor. Installs PersistentBase, PersistentRoot and CTP::Game vtables (the CTP::Game vtable also at +0x10), zeroes the member fields, builds… |
| `013794c0` | CTP::Game::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the CTP::Game destructor body 01378020, then frees the object with _free when bit 0 of the flag argument is set, and returns … |
| `0135f200` | CTP::Game::SetEntriesExceptArg | likely | Virtual slot [42] (thiscall, two stack arguments, returns void): walks the 4-byte-element vector at this+0x28..+0x2c; for each entry whose value from 01344e20 … |
| `0135f2b0` | CTP::Game::GetPlayAreaCardsDispatch | likely | Virtual slot [34] (thiscall, three stack arguments): traces Game::getPlayAreaCards(%d) with the id argument, then switches on the id: 0x38 and 0x3d build a car… |
| `0135f490` | CTP::Game::GetNodeTargetCardsDispatch | likely | Virtual slot [35] of CTP::Game (thiscall, three stack arguments): traces Game::getNodeTargetCards(%d) with the id, then switches on the id: 9 and 0x19 build a … |
| `0135fbb0` | CTP::Game::ForEachPlayAreaVirtual34 | likely | Virtual slot [81] of CTP::Game (thiscall, one stack argument, returns a bool dword): reserves storage through 0135aef0 sized from the argument's element count … |
| `0135fff0` | CTP::Game::SubmitOrReleaseDisplayAction | likely | Virtual slot [85] of CTP::Game (thiscall, one stack argument = object pointer, returns void): increments the global counter at 0x1cbdc4 (DAT_01cbdbc4). If the … |
| `01360780` | CTP::Game::FillTempPointerVector95 | likely | Virtual slot [95] of CTP::Game (fastcall, ECX = object, no stack arguments, returns void): zeroes a three-word temporary pointer vector, calls virtual +0x178 o… |
| `01360800` | CTP::Game::SetEntry30c | likely | Virtual slot [17] of CTP::Game (thiscall, two stack arguments = key by address and value, ret 8): calls 0051ff49 with ECX = object+0x30c and the key address to… |
| `01360960` | CTP::Game::CollectFilteredEntries98 | likely | Virtual slot [98] of CTP::Game (thiscall, five stack arguments, ret 0x14, returns a bool in AL): when 013837b0 reports false it walks a pointer vector held in … |
| `01360c90` | CTP::Game::UpdateAttributes | likely | Virtual slot 100 of CTP::Game. Traces 'Game::updateAttributes' and increments the dword at this+0x328. Then, over the same slot-81 element list, for each eleme… |
| `01362d00` | CTP::Game::AddObserver | likely | Virtual slot 114 of CTP::Game. Traces 'Adding Observer to Game: %d' with the argument (013508b0 and the '#' variant through 013541f0), allocates 0x4c bytes thr… |
| `0136dbd0` | CTP::Game::Setup | likely | Logs "gameIsSetup == true" and returns when the setup flag byte at this+0x17c is already 1. Otherwise it stores the setup time (param_2, or __time64 when zero)… |
| `01370df0` | CTP::Game::CardDeselected | likely | Asserts mCurrentTurn (line 0xfc6). Logs 'cardDeselected\t%d\t%d' (013541f0). It calls 013f7f70(param_2, param_3) to fetch the state machine and then calls 013e… |
| `01378ed0` | CTP::Game::DtorAdjustor_0x10 | likely | Adjustor thunk: subtracts 0x10 from ECX and jumps to 013794c0, the scalar deleting destructor. Decompiled as a call to FUN_013794c0. |

**CWAIController** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014050d0` | CWAIController::handleScript | confirmed | Runs one script step of the AI controller: traces 'CWAIController::handleScript %d' with the step index at this+0x28, asserts this+0x1c (aiValues) is non-null,… |
| `014059d0` | CWAIController::getInput | confirmed | Traces 'CWAIController::getInput' and the turn number from 013f7ed0. If this+0x25 (waitForUser) is set it traces 'CWAIController::getInput waitForUser' and ret… |

**CWArchetype** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01452730` | CWArchetype::CWArchetype | confirmed | Fastcall ctor with no stack arguments: calls base ctor FUN_01380760 on this, then stores CWArchetype::vftable (018ed40c) into [this] and the secondary CWArchet… |
| `014527b0` | CWArchetype::GetClassName | confirmed | Returns the address of the literal CWArchetype (mov eax, literal; ret), no arguments. |
| `014528f0` | CWArchetype::AdjustorThunkDtor | confirmed | Subtracts 4 from ECX (83 e9 04) and jumps to 01452980 (e9 88 00 00 00); a this-adjustor thunk for the secondary vtable. |
| `01452980` | CWArchetype::~CWArchetype | confirmed | Primary scalar deleting destructor shape: calls 014527f0 with this in ECX, then calls _free(this) only when bit 0 of its stack byte argument is set; returns th… |
| `014527c0` | CWArchetype::GetGlobal_01cbe4fc | likely | Returns the current value of global DAT_01cbe4fc (mov eax,[01cbe4fc]; ret); no this access. |
| `014527d0` | CWArchetype::SetGlobal_01cbe4fc | likely | Stores its single stack argument into global DAT_01cbe4fc and returns with ret 4; no this access. |
| `014527f0` | CWArchetype::DestructBody | likely | Fastcall (this in ECX): stores CWArchetype::vftable (018ed40c) into [this] and the secondary vftable (018ed3f8) into [this+4], then calls base body FUN_013803c… |
| `014528a0` | CWArchetype::SerializeStateWithTrace | likely | Thiscall with one stack argument (ret 4): if the argument is 0 obtains a stream from virtual slot [1]() else from slot [2](arg); traces Starting CWArchetype, c… |

**CWCard** (21)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0140c990` | CWCard::MoveToCardPosition | confirmed | Logs 'moveToCardPosition cardArchetype...' through 013508b0, inspects the card's hand, limbo, discard and parent placement through vslot calls, logs 'myHandPil… |
| `0140ce70` | CWCard::AttachTo | confirmed | Dynamic-casts the argument to Card (00d8d382). When the cast fails it returns this->vslot[74](arg). Otherwise it logs 'ZPCard::attachTo card: %d, dest: %d', ca… |
| `0140d010` | CWCard::CheckInstanceData | confirmed | Calls this->vslot[122](); when that does not return 1: if the instance-data pointer (+0x64) is non-null it calls its vslot[26](arg) and returns 0 on failure, e… |
| `0140d560` | CWCard::ValidateCommandObjectPreconditions | confirmed | Logs 'CWCard::validateCommandObjectPreconditions', then walks the command-object vector (begin +4, end +8) of the argument: for each element it calls its vslot… |
| `0140d730` | CWCard::ClearUseMapEntries | confirmed | When arg2 is 0x222f6 it checks the player and action pair (013402b0, 013457b0) and may return early, then logs '%d EQCard::clearUseMap EventUseLimit: %d', walk… |
| `0140d950` | CWCard::CheckCosts | confirmed | Logs 'ZPCard::checkCosts %d, %d'. Logs 'Is automatic' and returns 1 when 01322230 is true; logs 'Skip Cost' and returns 1 when 01383520(0x3d) is true. Otherwis… |
| `0140e7c0` | CWCard::GetValidTargetsForDrag | confirmed | Logs 'ZPCard::getValidTargetsFroDragWithEnv'; enumerates candidates via vslot[69](&list, 3) and tests each with vslot[55](target, env, 1) ('true == canbeplayed… |
| `0140ebe0` | CWCard::IsValidAction | confirmed | Logs 'CWCard::isValidAction %d action: %d (%d)' and returns 0 with a logged reason when: the game is ended (01348640), the action is on a card in play, the car… |
| `0140f1a0` | CWCard::IncrementUseLimit | confirmed | Logs the increment; when 01394b10(arg1) is non-zero, for id 0x222f6 it looks the key up in the list at this+0x7c and sets the counter to 1 when the lookup retu… |
| `0140fde0` | CWCard::DtorBody | confirmed | Stores the four CWCard vtable pointers into this (+0, +8, +0x48, +0x58), calls FUN_01380350 on the member at this+0x144, then calls the base cleanup FUN_0139ba… |
| `0140fe80` | CWCard::TypeName | confirmed | Virtual slot [11] of the CWCard secondary vtable (+8). Returns the address of the literal string 'CWCard' (no state read). |
| `0140ff00` | CWCard::CWCard | confirmed | Calls the base constructor FUN_0139d010 on this, stores the four CWCard vtable pointers (+0, +8, +0x48, +0x58), calls FUN_013806f0 on the member at this+0x144,… |
| `0140ff90` | CWCard::ScalarDeletingDtor | confirmed | Calls the destructor body 0140fde0 on this, frees this through the _free thunk when bit 0 of the flag argument is set, and returns this (ret 4). |
| `0140fe90` | CWCard::GetStaticDword | likely | Virtual slot [12] of the CWCard secondary vtable (+8). Returns the dword stored at global DAT_01cbe248 in EAX. |
| `0140fea0` | CWCard::SetStaticDword | likely | Virtual slot [13] of the CWCard secondary vtable (+8). Stores its single stack argument into global DAT_01cbe248 and returns with ret 4. |
| `0140fec0` | CWCard::AdjustorThunk_Sub8 | likely | Secondary-base adjustor for vtable +0x8 slot [0]: sub ecx,8 then jmp 0140ff90 (CWCard scalar deleting destructor). It does not call or free anything itself. |
| `0140fed0` | CWCard::AdjustorThunk_Sub48 | likely | Secondary-base adjustor for vtable +0x48 slot [0]: sub ecx,0x48 then jmp 0140ff90 (CWCard scalar deleting destructor). |
| `0140fee0` | CWCard::AdjustorThunk_Sub58 | likely | Secondary-base adjustor for vtable +0x58 slot [0]: sub ecx,0x58 then jmp 0140ff90 (CWCard scalar deleting destructor). |
| `0140fef0` | CWCard::AdjustorThunk_Sub58_Slot1 | likely | Secondary-base adjustor for vtable +0x58 slot [1]: sub ecx,0x58 then jmp 0140dc50 (not a destructor; the target is outside this shard). |
| `0140ffb0` | CWCard::New | likely | Allocates 0x150 bytes through operator new (00d833a6); if the pointer is non-null it constructs CWCard there through 0140ff00 and returns the object pointer pl… |
| `01410030` | CWCard::RegisterFactory | likely | Static initializer: registers the CWCard factory FUN_0140ffb0 under classID 0x222e1 by calling FUN_013ce130(0x222e1, factory) and FUN_013ce230(0x222e1, factory… |

**CWDeck** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01409230` | CWDeck::ExportText | confirmed | Virtual slot 27 (+0x6c) of CWDeck::vftable. Writes the deck name string at this+0x4 (file name) to a file opened with _fopen 'w'; on failure traces "Couldn't o… |

**CWDeckBuilderUtilities** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013eb8f0` | CWDeckBuilderUtilities::FormatName | confirmed | Virtual slot 4 with a hidden string return. Maps format id 1 to Standard, 0x1adb1 to Standard Draft, 0x1adb2 to Highlander, 0x1adb3 to Common Ground, 0x1adb5 t… |
| `013eb9d0` | CWDeckBuilderUtilities::FormatDescription | confirmed | Virtual slot 5 with a hidden string return. Returns the rules text for format id 1 (Standard), 0x1adb2 (Highlander), 0x1adb3 (Common Ground) and 0x1adb5 (No Ra… |
| `013ebe50` | CWDeckBuilderUtilities::CheckOwnedCards | confirmed | Virtual slot 1. dynamic_casts its deck argument from Deck to CWDeck (assert with aaDeck at line 0x38 on failure), reports the message You do not own the follow… |
| `013ed580` | CWDeckBuilderUtilities::ValidateFormat | confirmed | Virtual slot 3. Dynamic-casts the deck argument from Deck to CWDeck (error text Attempted to validate non ZPDeck through 012f5d80 when null), then dispatches o… |
| `013eaa90` | CWDeckBuilderUtilities::CWDeckBuilderUtilities | likely | Constructor body: calls 012dbb30 for base initialisation, then stores the CWDeckBuilderUtilities vftable at *this and returns this. |

**CWGame** (31)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0132bfc0` | CWGame::getNodeTargetCards | confirmed | Virtual (CWGame vtable slot 35; also CWMainController slot 35): traces 'CWGame::getNodeTargetCards(%d)' with the id through 013846b0; for any id other than 0x1… |
| `0132e200` | CWGame::getPlayArea | confirmed | Thiscall virtual (CWGame vtable slot 129; also CWMainController slot 129): traces 'CWGame::getPlayArea %d' with the id through 013846b0. Id 0x1e takes the Oppo… |
| `0132e320` | CWGame::getNodeTarget | confirmed | Thiscall virtual (CWGame vtable slot 128; also CWMainController slot 128): traces 'CWGame::getNodeTarget %d' with the id. Ids 0x1ae02 and 0x3e first test the n… |
| `0132fd50` | CWGame::DumpPlayerDrawDeck | confirmed | Thiscall virtual method of CWGame (vtable slot 122, also CWMainController slot 122): sprintf's 'Player%d Deck:' from the player id (param_2) and '%s DrawDeck:'… |
| `0132fe50` | CWGame::calculateCostMod | confirmed | Thiscall (this = param_1): logs 'CWGame::calculateCostMod ' through 01384af0, then walks the 4-byte id vector at this+0x3b0..+0x3b4; for each id it resolves an… |
| `013310a0` | CWGame::BroadcastToEntriesAt28 | confirmed | Thiscall virtual method of CWGame (vtable slot 83, also CWMainController slot 83): walks the 4-byte entries from this+0x28 to this+0x2c, resolves each entry th… |
| `01331120` | CWGame::MatchWonByGameCount | confirmed | Thiscall predicate: returns 1 at once when the match flag at this+0x34c is neither 2 nor 3; otherwise it walks the list whose head is at this+0x3d0 and for eac… |
| `01331210` | CWGame::ForEachElementCallSlot150 | confirmed | Thiscall virtual method of CWGame (vtable slot 152, also CWMainController slot 152): calls its own virtual slot 151 (+0x25c) with no argument, then for each 4-… |
| `01332260` | CWGame::DeleteOwnedEntriesAt0x3dc | confirmed | Thiscall virtual method of CWGame (vtable slot 151, also CWMainController slot 151): for each pointer in the vector at this+0x3dc..+0x3e0 it calls the object's… |
| `01332c90` | CWGame::StopGame | confirmed | Thiscall virtual method of CWGame (vtable slot 38, also CWMainController slot 38): logs 'STOP CALLED ON THE GAME' through 013508b0 and 'stop' through 013541f0,… |
| `01332ed0` | CWGame::DeliverGameEndNotice | confirmed | Thiscall (param_1 = game, param_2 = end reason code, param_3 = winner, param_4 = loser, asserted non-null): casts both players to CWPlayer (00d8d382) and the l… |
| `01333470` | CWGame::Write | confirmed | Thiscall virtual method at slot 10 of the secondary CWGame interface (CWGame::vftable +0x10 and CWMainController +0x10): with a null writer argument it obtains… |
| `013356b0` | CWGame::SetupPlayerDeckOrDefault | confirmed | Thiscall virtual method of CWGame (vtable slot 14, also CWMainController slot 14): when the player argument is non-null and the game word at +0x288 equals 0x1a… |
| `013358f0` | CWGame::EndAsDraw | confirmed | Virtual slot 119. If callee 01347ab0 returns false it logs 'GAME ENDS IN A DRAW', fills a local id vector through 013630f0, calls 013486d0(this, id, draw text)… |
| `01335a10` | CWGame::EndAsAdjournment | confirmed | Virtual slot 27. If callee 01347ab0 returns false it logs 'GAME ENDS IN ADJOURNMENT', fills a local id vector through 013630f0, calls 013486d0(this, id, adjour… |
| `01335b30` | CWGame::UpdateAttributes | confirmed | Virtual slot 100. Logs 'Game::updateAttributes' and calls 01348d70. Only when virtual slot 81 (+0x144) returns true: walks the element vector at +0x28..+0x2c a… |
| `01336d20` | CWGame::Setup | confirmed | Virtual slot 10. If the setup byte at +0x17c is set it logs 'gameIsSetup == true' and returns. Otherwise it logs the seed ('CW_ERROR: NO SEED PASSED IN!' for 0… |
| `01337510` | CWGame::DispatchTextCommand | confirmed | Virtual slot 146. Returns for a null line or one starting with '#'. Otherwise copies the line into a std::string and tests it against the keywords (setup, matc… |
| `01339080` | CWGame::GetActiveCards | confirmed | Virtual slot 60. Logs 'getActiveCards Current Event: %d, %d, %d' with the phase (slot 15 of the CWGameTurn at +0x68) and the event (013f7eb0). Looks up the pla… |
| `01339370` | CWGame::GetActiveActionsAndCards | confirmed | Virtual slot 61. Reads the phase through slot 15 of the CWGameTurn at +0x68 and the event through 013f7eb0, logs 'CWGame::getActiveActionsAndCards. phase: %d e… |
| `0133ca80` | CWGame::GameWonFromLossByPlayer | confirmed | Thiscall taking a player object (param_2), a player id (param_3) and a flag (param_4). When FUN_01347ab0 and FUN_01348640 are both false it logs 'GAME WON FROM… |
| `0133d480` | CWGame::~CWGame | confirmed | CWGame destructor body (thiscall). Re-installs the CWGame vtables at +0 and +0x10, logs 'KILLING CWGame VIA DESTRUCTOR' through FUN_013508b0, calls FUN_0124731… |
| `0133d5c0` | CWGame::GameLostByPlayer | confirmed | Virtual slot 44 of CWGame (also CWMainController), with a player id (param_2) and a reason code (param_3). If FUN_01353b80(player) is clear it logs 'GAME LOST … |
| `0133d900` | CWGame::findPlayerDeck | confirmed | Virtual slot 149 of CWGame (also CWMainController), taking a player argument (param_2). It logs 'CWGame.findPlayerDeck - %d', then walks the deck list between … |
| `0133e060` | CWGame::CWGame | confirmed | Thiscall constructor of CWGame: calls FUN_01378f50 on its argument, installs the CWGame vtables at +0 and +0x10, calls FUN_005258fb to construct a member, clea… |
| `0133e180` | CWGame::DeletingDtor | confirmed | Scalar deleting destructor of CWGame (vtable slot 0 of CWGame::vftable): saves this in ESI, calls the destructor body FUN_0133d480, then when bit 0 of the flag… |
| `0132bdd0` | CWGame::CreateCWCard | likely | Thiscall virtual (CWGame vtable slot 4; the same slot of CWMainController): calls operator new 00d833a6 with 0x150 bytes; when the result is non-null it calls … |
| `0132c090` | CWGame::CardSelected | likely | Virtual (CWGame vtable slot 52; CWMainController falls through to it): when the byte at this+0x3f4 is set it clears it, allocates 0x14 bytes (00d833a6), initia… |
| `01331bd0` | CWGame::RemoveModifier | likely | Thiscall (this = param_1, modifier = param_2): when the modifier pointer is null it asserts through 012f5a60 with 'modifier'; finds the id of the modifier (013… |
| `0133f420` | CWGame::TallyDeckCardIds | likely | If 01350660(param_2) is non-null and 01344f00 dynamic_casts it to CWDeck, it calls the deck's vtable +0x4c to fill a temporary int vector, then counts copies o… |
| `0133f740` | CWGame::InsertKeyedRecordIfAbsent | likely | Looks up int key param_2 in the ordered map at this+0x200 (01331340). When the key is absent and param_4 is 1 (with the stack flag in_stack_00000040 clear), it… |

**CWGameTurn** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01410130` | CWGameTurn::CanDrawCards | confirmed | Virtual slot [17] of the CWGameTurn vtable (018e56c4). Logs 'canDrawCards %d, %d' through FUN_013508b0 on this+4 with its two arguments, then calls FUN_0135066… |
| `01410460` | CWGameTurn::advancePhase | confirmed | Virtual slot [20] of the CWGameTurn vtable (018e56c4). Sets this+0x10 to 0x1b. If the byte at this+0x7c is 1 it copies this+0x80 into this+0xc, calls vtable sl… |
| `01410b30` | CWGameTurn::TypeName | confirmed | Virtual slot [11] of the CWGameTurn vtable (018e56c4): returns the address of the literal 'CWGameTurn' (6-byte body: mov eax, imm; ret). |
| `01410de0` | CWGameTurn::ScalarDeletingDtor | confirmed | Calls the CWGameTurn destructor body 01410aa0 on this, frees this through the _free thunk when bit 0 of the flag argument is set, and returns this (ret 4). |
| `014101f0` | CWGameTurn::SetPlayerAndNotify | likely | Asserts the player argument is non-null (cwgameturn.cpp line 0x124), stores FUN_01344e20() into this+0xa8 (param_1[0x2a]), calls this vtable+0x48 with the play… |
| `01410340` | CWGameTurn::DispatchEventCode | likely | Virtual slot [19] of the CWGameTurn vtable (018e56c4). Sets this+0x10 to 0x1b and this+0x88 to 0, then branches on the event code argument: 0x29 stores it in t… |
| `01410890` | CWGameTurn::PushTurnDescriptionEvent | likely | Virtual slot [24] of the CWGameTurn vtable (018e56c4). Allocates a 0x14-byte object (FUN_00d833a6), initialises it with FUN_012f9eb0, puts event code 0x15 thro… |
| `01410b40` | CWGameTurn::GetStaticDword | likely | Virtual slot [12] of the CWGameTurn vtable (018e56c4): returns the dword at global DAT_01cbe24c (a different global from the one used by CWCard). |
| `01410b50` | CWGameTurn::SetStaticDword | likely | Virtual slot [13] of the CWGameTurn vtable (018e56c4): stores its stack argument into global DAT_01cbe24c and returns with ret 4. |
| `01410b70` | CWGameTurn::GenerateEvent | likely | Virtual slot [21] of the CWGameTurn vtable (018e56c4). Stores its argument as the event code at this+0x10, logs 'GENERATEEVENT: %d' through FUN_013508b0 on thi… |
| `01410e00` | CWGameTurn::New | likely | Allocates 0xb8 bytes through operator new (00d833a6); when the allocation is non-null it runs the constructor 01410d70 on it and returns the object, otherwise … |
| `01410e70` | CWGameTurn::RegisterFactory | likely | Static initializer: registers the CWGameTurn factory 01410e00 under classID 0x222e4 by calling FUN_013ce130 and then FUN_013ce230 with the same key and factory. |

**CWMainController** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01315690` | CWMainController::Dtor | confirmed | Destructor body (not the scalar deleting wrapper). Stores CWMainController::vftable at +0 and at +0x10 (018d720c). Deletes the owned objects at +0x410 and +0x4… |
| `01315ab0` | CWMainController::ScalarDeletingDtor | confirmed | Vtable slot [0] of CWMainController: calls the destructor body 01315690, then frees the object when flag bit 0 of param_2 is set; returns the object pointer. |
| `0131a7f0` | CWMainController::IdGatedStepOrFallback | likely | Virtual slot [57] of CWMainController, thiscall with three arguments and a byte result. When the object at this+0x40c is non-null and 014029e0 equals param_2, … |
| `0131a900` | CWMainController::IdGatedValueOrFallback | likely | Virtual slot [51] of CWMainController, thiscall with five stack arguments (ret 0x14). Same gate as 0131a7f0 (this+0x40c non-null and 014029e0 equal to param_2)… |

**CWPlayer** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014186c0` | CWPlayer::SetupDeck | confirmed | Virtual slot 15 of CWPlayer::vftable. Thiscall with one stack argument (a Deck object). Looks up a name through FUN_00f942a0 and only continues when it returns… |
| `01418ae0` | CWPlayer::GetTypeName | confirmed | Virtual slot 11 of CWPlayer::vftable. Returns the address of the text CWPlayer (bytes b8 imm32 c3, mov eax then ret). |
| `01418b20` | CWPlayer::CWPlayer | confirmed | Fastcall with ECX = this. Runs FUN_01345ef0 under an SEH frame, stores CWPlayer::vftable into this, zeroes this+0x4c, calls FUN_01247380, zeroes this+0x5c and … |
| `01418ba0` | CWPlayer::ScalarDeletingDtor | confirmed | Virtual slot 0 of CWPlayer::vftable with the scalar deleting shape: with ECX = this it calls the destructor body 01418a60, tests bit 0 of the stack flag byte, … |
| `01418350` | CWPlayer::Deserialize | likely | Virtual slot 9 of CWPlayer::vftable. Calls the virtual at vtable+0x10 on this with the value (must return nonzero, else it reports false at cwplayer.cpp line 0… |
| `014184c0` | CWPlayer::CreateHolderCard | likely | Virtual slot 14 of CWPlayer::vftable. Allocates 0x150 bytes and builds an object with FUN_0140ff00 that is stored at this+0x30, then calls its virtuals at +0x4… |
| `01418990` | CWPlayer::TopUpTo6 | likely | Virtual slot 31 of CWPlayer::vftable, fastcall with ECX = this. Gets an object with FUN_0139d7c0 and calls its virtual at +0x34 with a local vector start. It c… |
| `01418a60` | CWPlayer::~CWPlayer | likely | Fastcall with ECX = this. Stores CWPlayer::vftable into this, calls FUN_01247310 under an SEH frame, then calls FUN_01345d50 and returns nothing. It is the des… |
| `01418af0` | CWPlayer::GetStaticDword | likely | Virtual slot 12 of CWPlayer::vftable. Returns the dword in the global DAT_01cbe258 (mov eax,[abs]; ret). |
| `01418b00` | CWPlayer::SetStaticDword | likely | Virtual slot 13 of CWPlayer::vftable. Stores its one stack argument into the global DAT_01cbe258 and returns with ret 4. |

**CWPlayerPlayArea** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `015156d0` | CWPlayerPlayArea::GetTypeName | confirmed | Virtual slot 11 of the secondary CWPlayerPlayArea vtable. Returns the address of the literal CWPlayerPlayArea (mov eax, imm32; ret). |
| `01515710` | CWPlayerPlayArea::~CWPlayerPlayArea | confirmed | Stores both CWPlayerPlayArea vtables, and when the pointer at +0x54 is nonnull deletes that object through its vtable slot 0 with flag 1 (after FUN_01344cd0). … |
| `01515880` | CWPlayerPlayArea::ReadState | confirmed | Virtual slot 9 of the secondary CWPlayerPlayArea vtable. Calls vtbl+0x10 with the argument and an output pointer; on failure it reports false at cwplayerplayar… |
| `015159c0` | CWPlayerPlayArea::ScalarDeletingDtor | confirmed | Virtual slot 0 of the primary CWPlayerPlayArea vtable. Calls FUN_01515710 (the destructor body), then calls _free on this when bit 0 of the second argument is … |
| `01515650` | CWPlayerPlayArea::CWPlayerPlayArea | likely | Calls FUN_0139d4f0 on this, stores CWPlayerPlayArea::vftable into *this and into the second base slot at +8, clears the dword at +0x3c, and returns this. |
| `01515790` | CWPlayerPlayArea::Init | likely | Virtual slot 16 of CWPlayerPlayArea. Allocates 0x48 bytes with FUN_00d833a6; if nonnull constructs the object with FUN_0139d3d0 and stores it at this+0x54, els… |
| `015158e0` | CWPlayerPlayArea::WriteState | likely | Virtual slot 10 of the secondary CWPlayerPlayArea vtable. Calls vtbl+0x08 with the argument, calls FUN_0139de80 on the argument, then calls vtbl+0x0c with it a… |
| `01515910` | CWPlayerPlayArea::ScalarDeletingDtorAdjustor | likely | Thunk for the secondary vtable slot 0: subtracts 8 from ECX (this) and jumps to FUN_015159c0 (the scalar deleting destructor of the primary sub-object). |
| `01515920` | CWPlayerPlayArea::Create | likely | Allocates 0x58 bytes with FUN_00d833a6; when nonnull constructs the object with FUN_01515650 and returns the object address plus 8 (the +8 sub-object); returns… |
| `015159e0` | CWPlayerPlayArea::Enter | likely | Thiscall with a trace: formats Entering ZPPlayerPlayArea through FUN_01385040, asserts the valueData argument (FUN_012f5a60 at cwplayerplayarea.cpp line 0x49) … |

**CWSumExpression** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014540e0` | CWSumExpression::CWSumExpression | confirmed | Fastcall ctor with no stack arguments: calls base ctor FUN_0148dcd0 on this, then stores CWSumExpression::vftable (018ed76c) into [this]; returns this. |
| `01454160` | CWSumExpression::GetClassName | confirmed | Returns the address of the literal CWSumExpression (mov eax, literal; ret), no arguments. |
| `01454300` | CWSumExpression::~CWSumExpression | confirmed | Scalar deleting destructor shape: stores CWSumExpression::vftable into [this], calls base body FUN_0148dd90 with this in ECX, then calls _free(this) only when … |
| `01454170` | CWSumExpression::GetGlobal_01cbe508 | likely | Returns the current value of global DAT_01cbe508 (mov eax,[01cbe508]; ret); no this access. |
| `01454180` | CWSumExpression::SetGlobal_01cbe508 | likely | Stores its single stack argument into global DAT_01cbe508 and returns with ret 4; no this access. |
| `014541a0` | CWSumExpression::DestructBody | likely | Fastcall (this in ECX): stores CWSumExpression::vftable into [this], then calls the base body FUN_0148dd90 with this in ECX; takes no flag argument and does no… |
| `01454230` | CWSumExpression::SerializeWithTrace | likely | Thiscall with one stack argument: calls virtual slot [2](arg), traces Starting CWSumExpression with 013c26e0, calls base 0148df30(arg), traces Ending CWSumExpr… |

**CWTargetQueryNode** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014537c0` | CWTargetQueryNode::CWTargetQueryNode | confirmed | Fastcall ctor with no stack arguments: calls base ctor FUN_01489270 on this, then stores CWTargetQueryNode::vftable (018ed61c) into [this]; returns this. |
| `01453840` | CWTargetQueryNode::GetClassName | confirmed | Returns the address of the literal CWTargetQueryNode (mov eax, literal; ret), no arguments. |
| `014539f0` | CWTargetQueryNode::~CWTargetQueryNode | confirmed | Scalar deleting destructor shape: stores CWTargetQueryNode::vftable into [this], calls base body FUN_01489340 with this in ECX, then calls _free(this) only whe… |
| `01453850` | CWTargetQueryNode::GetGlobal_01cbe504 | likely | Returns the current value of global DAT_01cbe504 (mov eax,[01cbe504]; ret); no this access. |
| `01453860` | CWTargetQueryNode::SetGlobal_01cbe504 | likely | Stores its single stack argument into global DAT_01cbe504 and returns with ret 4; no this access. |
| `01453880` | CWTargetQueryNode::DestructBody | likely | Fastcall (this in ECX): stores CWTargetQueryNode::vftable into [this], then calls the base body FUN_01489340 with this in ECX; takes no flag argument and does … |
| `01453920` | CWTargetQueryNode::SerializeWithTrace | likely | Thiscall with one stack argument: calls virtual slot [2](arg), traces Starting CWTargetQueryNode with 013c26e0, calls base 014895e0(arg), traces Ending CWTarge… |

**CWValueNode** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014534f0` | CWValueNode::CWValueNode | confirmed | Fastcall ctor with no stack arguments: calls base ctor FUN_014882c0 on this, then stores CWValueNode::vftable (018ed55c) into [this]; returns this. |
| `01453570` | CWValueNode::GetClassName | confirmed | Returns the address of the literal CWValueNode (mov eax, literal; ret), no arguments. |
| `01453760` | CWValueNode::~CWValueNode | confirmed | Scalar deleting destructor shape: stores CWValueNode::vftable into [this], calls base body FUN_01488390 with this in ECX, then calls _free(this) only when bit … |
| `01453580` | CWValueNode::GetGlobal_01cbe500 | likely | Returns the current value of global DAT_01cbe500 (mov eax,[01cbe500]; ret); no this access. |
| `01453590` | CWValueNode::SetGlobal_01cbe500 | likely | Stores its single stack argument into global DAT_01cbe500 and returns with ret 4; no this access. |
| `014535b0` | CWValueNode::DestructBody | likely | Fastcall (this in ECX): stores CWValueNode::vftable into [this], then calls the base body FUN_01488390 with this in ECX; takes no flag argument and does not fr… |
| `01453690` | CWValueNode::SerializeWithTrace | likely | Thiscall with one stack argument: calls virtual slot [2](arg), traces Starting CWValueNode with 013c26e0, calls base 01488880(arg), traces Ending CWValueNode, … |

**Card** (48)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01341070` | Card::HasSuppressEffect | confirmed | Thiscall on a Card with a ValueData param_2. Traces 'hasSuppressEffect(valueData): %d (%d)' with the payload to the trace object at this+0x30. For type word 2 … |
| `0138a390` | Card::GetCardActionCostsTrace | confirmed | Virtual slot [61] of Card::vftable and CWCard::vftable (thiscall, two stack arguments, ret 8). Logs 'getCardActionCosts' through 013508b0 on the object at this… |
| `0138a3d0` | Card::EvaluateActionCosts | confirmed | Virtual slot [62] of Card::vftable and CWCard::vftable (thiscall, two stack arguments, ret 8). Gets an evaluator object from 01322160 (returns 1 when null), sa… |
| `0138a570` | Card::GetFlag69 | confirmed | Virtual slot [18] of Card::vftable and CWCard::vftable (fastcall). Returns the byte at this+0x69 (mov al,[ecx+0x69]; ret). |
| `0138a580` | Card::SetFlag69 | confirmed | Virtual slot [47] of Card::vftable and CWCard::vftable (thiscall, one byte argument, ret 4). Stores the byte argument into this+0x69. |
| `0138aac0` | Card::AddKeywordByValueType | confirmed | Virtual slot [108] of Card::vftable and CWCard::vftable (thiscall, one stack argument, a ValueData). If its type tag is 2 it calls virtual slot 107 (+0x1ac) wi… |
| `0138ab40` | Card::RemoveKeywordByValueType | confirmed | Virtual slot [110] of Card::vftable and CWCard::vftable (thiscall, one stack argument, a ValueData). If its type tag is 2 it calls virtual slot 109 (+0x1b4) wi… |
| `0138ad80` | Card::SetPropertyOrForward | confirmed | Virtual slot [1] of Card::vftable (thiscall, two stack arguments, returns a byte). Asserts this+4 (mPropertySet) non-null (HasProperties.h line 0x30). It calls… |
| `0138ae00` | Card::GetPropertyOrArchetype | confirmed | Virtual slot [3] of Card::vftable and CWCard::vftable (thiscall, two stack arguments, returns a byte). Asserts the archetype object at +0x64 (mArchetype, Card.… |
| `0138aeb0` | Card::ForwardToArchetypeSlot5 | confirmed | Virtual slot [5] of Card::vftable and CWCard::vftable (thiscall, one stack argument, void). When 014240c0 does not take the argument and the archetype object a… |
| `0138b190` | Card::MoveToDiscardPile | confirmed | Virtual slot [82] of Card::vftable and CWCard::vftable (fastcall). Sets the guard byte at this+0x6a to 1, resolves the card's player with 01350660(this+0x40) (… |
| `0138b2d0` | Card::TryMoveToDiscardPile | confirmed | Virtual slot [84] of Card::vftable and CWCard::vftable (fastcall). Returns 0 when the guard byte at +0x6a is already 1. Otherwise it sets the guard, resolves t… |
| `0138b510` | Card::ForwardToArchetypeSlot21 | confirmed | Virtual slot [39] of Card::vftable and CWCard::vftable (fastcall). Asserts the archetype object at this+0x64 is non-null (Card.cpp line 0x116a), then calls its… |
| `01391e00` | Card::InstallKeyValue | confirmed | Card virtual slot 23 (key, ValueData value). If the value is type 5, it re-evaluates the expression through its vtable +0x38; on failure it logs 'ValueData Exp… |
| `01392d10` | Card::GameSpecificStub | confirmed | Card virtual slot 137. Calls 01300680 with 3 and assigns the constant 'GAME SPECIFIC IMPLEMENTATION REQUIRED' to the std::string at *(argument+8). |
| `01393c20` | Card::SetArchetypeId | confirmed | Card virtual slot 45 (archetype id). When the id differs from this+0x60: logs '*** Setting Arch %d on card %d' through 01340340 and 013508b0 when tracing is on… |
| `01394920` | Card::GetIntegerProperty | confirmed | Card virtual slot 87 (key, trace object). Special keys: 0x243 returns the value of 013f7ea0 after 01347e10; 0x9e returns the 01340370 value when the owner chec… |
| `01397e00` | Card::GetValidTargets | confirmed | Thiscall with two stack arguments (output target list, param_3), virtual at Card::vftable[56]. Logs 'Card::getValidTargets %s (%d)' (card name at this+0x128 an… |
| `01398680` | Card::GetCardActionEffects | confirmed | Thiscall with a stack argument and an output pointer, virtual at Card::vftable[64] (also CWCard::vftable[64]). Logs 'Card::getCardActionEffects %d (%d), %d' th… |
| `0139d1e0` | Card::ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor: calls the Card destructor body 0139ba50 on this, then when bit 0 of the stack flag argument is set calls _free(this) throu… |
| `0138a6e0` | Card::EvaluateInTemporaryEnvironment | likely | Virtual slot [59] of Card::vftable and CWCard::vftable (thiscall; the bytes end with ret 0xc, three stack-argument slots, while Ghidra shows two arguments). Co… |
| `0138a790` | Card::EvaluateInTemporaryEnvironmentB | likely | Virtual slot [57] of Card::vftable and CWCard::vftable (thiscall, two stack arguments, ret 8). Its bytes match 0138a6e0 except for the final ret immediate (8 h… |
| `0138ad30` | Card::ClearFlag69AndNotify | likely | Virtual slot [103] of Card::vftable and CWCard::vftable (fastcall). Clears the byte at this+0x69, calls virtual slot 99 (+0x18c) on this, then calls the virtua… |
| `0138d260` | Card::AddElementToPlayerLogged | likely | Virtual slot [16] of Card (Card::vftable and CWCard::vftable): stores param_2+0x20 at this+0x5c, calls 013402d0(param_3), stores param_2 at this+0x30 (log cont… |
| `0138df90` | Card::FilterAttrModsPass | likely | Virtual slot [140] of Card (CWCard too): walks the PlayElement vector at this+0xd0..0xd4; for each element that dynamic_casts to ValidAttrModFilter it calls 01… |
| `0138e060` | Card::CheckDynamicModifier | likely | Virtual slot [93] of Card (CWCard too), about 590 bytes: reads the value param_2 through vtbl[1] and accepts at once when the read succeeds, the type is 2 and … |
| `0138e2c0` | Card::CheckSuppressEffect | likely | Virtual slot [114] of Card (CWCard too), about 640 bytes: consumes the sign flag of param_2 via 013c5d10 (notFlag), requires instance data (this[0x19] or virtu… |
| `0138e710` | Card::HasKeyword | likely | Virtual slot [113] of Card: for a value of type 2 calls this->vtbl[112](value); for type 6 (integer list) returns 1 if any element passes vtbl[112]; for other … |
| `0138e7d0` | Card::ContainsKeywordKey0xdc | likely | Virtual slot [115] of Card (CWCard too): returns 0 unless vtbl[18]() is true; then obtains the id list from vtbl[133](0xdc) and for each element logs 'Got Keyw… |
| `0138f1b0` | Card::GetDynGameTextMod | likely | Virtual slot [128] of Card (CWCard too): when 012b8ff0(this+0x60, this+0x5c) finds a value and 01350660(this+0x40) is non-zero it reads the id list through the… |
| `0138f780` | Card::RemoveKeywordValue | likely | Virtual slot [109] of Card (CWCard too): logs 'Card::removeKeyword %d' with param_2, reads key 2 through vtbl[1] and requires an integer-list value (asserts at… |
| `0138fea0` | Card::ClearKeyedList | likely | Virtual slot [136] of Card (CWCard too): fetches the list for key param_2 through vtbl[133]; for key 0xdc it range-moves the list to empty (0138ed50); for any … |
| `01390950` | Card::MatchesMappedKeywordMod | likely | Virtual slot [38] of Card (CWCard too), returns bool: false when param_2 is null; consumes the sign flag of param_3 (013c5d30); walks the 0xdc-keyed mapped id … |
| `01390e00` | Card::ParseKeywordMatch | likely | Card virtual slot 22. Returns 0 unless both arguments are string ValueData (type 3). A leading '!' in the first string marks a negated keyword and is stripped.… |
| `01393bc0` | Card::SetPropertyValue | likely | Card virtual slot 4 (key, value). For key 0x1c it first copies the ValueData argument through 013010e0. It calls 01424050(key, value); when that returns 1 it c… |
| `01394e30` | Card::IncrementPairCount | likely | Card virtual slot 130 (two int key words). Looks up the (int,int) key in the map whose header is at this+0xec (end node at this+0xf0) through 0138eaa0. When pr… |
| `01395560` | Card::EvaluatePreventBy | likely | Thiscall with two stack arguments, virtual at Card::vftable[65] (also CWCard::vftable[65]). Calls 01394b10(this, param_2) for an id, then 01322210 with ECX = t… |
| `01395dd0` | Card::DebugDump | likely | Thiscall with one byte argument (verbose flag), virtual at Card::vftable[85] (also CWCard::vftable[85]). If the owner at this+0xc has flag +0x22c set, it write… |
| `01396310` | Card::RemoveValueAttrModFilter | likely | Thiscall with one stack argument, virtual at Card::vftable[119] (also CWCard::vftable[119]). Logs 'removing ValueAttr Mod Fitler from %s on %s' through 013508b… |
| `01396f60` | Card::DynamicKeywordRefresh | likely | Thiscall with one stack argument, virtual at Card::vftable[90] (also CWCard::vftable[90]). Does nothing when the argument is 0. When vtbl[+0x48] is set it gath… |
| `013972a0` | Card::AddAttributeModifier | likely | Thiscall with the attribute id in ECX-side argument position (param_2) and two stack arguments, virtual at Card::vftable[94] (also CWCard::vftable[94]). Writes… |
| `01397c10` | Card::RemoveAllModifierNodes | likely | Fastcall (ECX = this, no stack arguments), virtual at Card::vftable[99] (also CWCard::vftable[99]). Walks the node list at this+0xc4; for each node it walks th… |
| `01399e90` | Card::CollectModifierValues | likely | Virtual slot 117 of Card (offset 0x1d4). If vslot +0x48 of this returns true: runs the 013813d0/01381430/01350660/013815b0 accessor sequence, calls vslot +0x21… |
| `0139b1e0` | Card::MapSubscriptOrInsertDefault | likely | Virtual slot 133 of Card (offset 0x214). Looks up the key at *param_2 in the int-keyed map at this+0xc0 (header this+0xc0, head this+0xc4); when absent builds … |
| `0139b2f0` | Card::ResolveArchetypeList | likely | Virtual slot 138 of Card (offset 0x228), three arguments. Looks up each id from the list returned by 01434530 and 01326fd0 with 012b8ff0(id, this+0x5c); assert… |
| `0139ba10` | Card::ClearTreeF8 | likely | Virtual slot 32 of Card (offset 0x80). Erases the tree whose head pointer is at this+0xfc (calls 0137fbe0 with the head's link), then zeroes the size at this+0… |
| `0139ba50` | Card::DtorBody | likely | Thiscall destructor body: stores the Card vtables 018de574 (this), 018de538 (+8), 018de528 (+0x48) and 018de514 (+0x58), tears down the member containers throu… |
| `0140f310` | Card::GetAvailableActions | likely | Calls vtable+0x78 on this to fill a local list of action pairs; on failure logs 'couldn't get available actions in card.getAvailableActions' via FUN_013508b0 a… |

**ChainedActionState** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0147a970` | ChainedActionState::leaveState | confirmed | Virtual slot [18] (+0x48) of ChainedActionState. Traces 'ChainedActionState::leaveState' via 0145c4d0 and 013508b0. If any of 013836f0, 01383a20 (>0) or 013837… |
| `0147ab50` | ChainedActionState::sendCloseReveal | confirmed | Thiscall with one stack argument (bool param_2 = target gotten). Traces 'ChainedActionState::sendCloseReveal'. Walks the 4-byte reveal ids in the vector at thi… |
| `0147aee0` | ChainedActionState::~ChainedActionState | confirmed | Destructor body (__fastcall, void). Stores ChainedActionState::vftable at this, calls 01300cd0, releases three owned buffers (each pointer checked non-null, th… |
| `0147b030` | ChainedActionState::ScalarDeletingDtor | confirmed | Virtual slot [0] of ChainedActionState; scalar deleting destructor shape (30 bytes, ret 4): calls the destructor body 0147aee0 on this, tests bit 0 of the stac… |
| `0147b050` | ChainedActionState::ChainedActionState | confirmed | Default constructor (__fastcall, returns this). Calls base ctor 0147c980, stores ChainedActionState::vftable, zeroes this+0x100..0x110 slots, +0x118..0x120, +0… |
| `0147b130` | ChainedActionState::ChainedActionState | confirmed | Constructor taking one argument (param_2). Calls base ctor 0147c9f0 with param_2, stores ChainedActionState::vftable, zeroes the same containers and fields as … |
| `0147b430` | ChainedActionState::checkReturnValue | confirmed | Fastcall (ECX = this, returns 1, or 0 when there is no returnValueMap). Gets the returnValueMap (013f37e0). Sets this+0x128 when the 013836c0 target check pass… |

**ClientSimpleCardGameManager** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ad1590` | ClientSimpleCardGameManager::Ctor | confirmed | Calls the base manager constructor, installs two manager vtable pointers, zeros internal state fields, and stores two global defaults. |
| `00ad1a30` | StartManagerStateIfIdle | confirmed | When +0x40 is zero, sets state +0x2ba90 to 1, stores the current clock pair at +0x5ef50/+0x5ef54, and caches a global pointer at +0x28 if absent. |
| `00ad1a80` | HandleTcgLaunchTicket | confirmed | Accepts launch fields only in state 5, logs and returns otherwise, updates timed state when either first field is nonzero, dispatches campaign-start UI for pos… |
| `00ad1ef0` | ClientSimpleCardGameManager::Shutdown | confirmed | Logs tickets and station, removes a matching emitter and TcgMainWindow, clears the tree and references, resets the state timestamp, hides the TCG handler, and … |
| `00ad22c0` | ClientSimpleCardGameManager::Dtor | confirmed | Restores both manager vtable pointers, calls Shutdown, then runs the trading-card manager base destructor. |
| `00ad2360` | ClientSimpleCardGameManager::ScalarDeletingDtor | confirmed | Runs the manager destructor and frees the object when flag bit 0 is set; otherwise returns the object pointer. |
| `00ad2380` | AdvanceTcgStateMachineAndRecords | confirmed | Advances states 1 through 5, triggers asset/UI callbacks and elapsed-time checks, sends leave/sub-opcode requests through shared helpers, then walks and remove… |
| `00ad2d30` | ClientSimpleCardGameManager::InitCardClient | confirmed | Clears a ready flag, announces UI unlock, creates the TCG interface and Flash view, loads TradingCardMain/TCCW_Main.swf, logs realm and tickets, shows the TCG … |
| `00ad3150` | StartTcgClientInitialization | confirmed | Sets state +0x2ba90 to 3, stores the current 64-bit clock pair, and calls the client initialization routine. |

**CloneState** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01479ea0` | CloneState::~CloneState | confirmed | Destructor body (void, no free). Stores CloneState::vftable at this, then calls 0147c520 (the CancellableStateMachineState destructor body: stores its own vtab… |
| `01479f10` | CloneState::GetTypeName | confirmed | Virtual slot [11] of CloneState. Returns the constant string pointer 0x018F3ED8 ("CloneState"); 6 bytes, no arguments. |
| `01479f50` | CloneState::enterState | confirmed | Virtual slot [14] (+0x38) of CloneState. Traces 'CloneState::enterState' via 0145c4d0/013508b0, calls this's vtable +0x4c, and reads the returnValueMap (013f37… |
| `0147a580` | CloneState::restoreState | confirmed | Virtual slot [17] (+0x44) of CloneState. Traces 'CloneState::restoreState' via 0145c4d0 and 013508b0, then returns 2 (25 bytes). |
| `0147a5a0` | CloneState::leaveState | confirmed | Virtual slot [18] (+0x48) of CloneState. Traces 'CloneState::leaveState' via 0145c4d0 and 013508b0, calls 013f39c0 (no arguments, unnamed), and returns 1 (34 b… |
| `0147a660` | CloneState::SerializeIds | confirmed | Virtual slot [10] (+0x28) of CloneState. Obtains a stream from vtable slot +0x4 (no argument) or +0x8 (argument param_2) and passes text 'Starting CloneState' … |
| `0147a720` | CloneState::ScalarDeletingDtor | confirmed | Virtual slot [0] of CloneState; the scalar deleting destructor shape: stores CloneState::vftable at this, calls 0147c520 (base destructor body), tests bit 0 of… |
| `0147a780` | CloneState::CloneState | confirmed | Default constructor (__fastcall, returns this). Calls base ctor 0147c980, stores CloneState::vftable at this, assigns the 10-char string 'CloneState' to this+0… |
| `0147a800` | CloneState::CloneState | confirmed | Constructor taking one argument (param_2). Calls base ctor 0147c9f0 with param_2, stores CloneState::vftable, assigns 'CloneState' (10 chars) to this+0x1C, zer… |
| `01479f20` | CloneState::GetStaticValue | likely | Virtual slot [12] of CloneState. Returns the dword global DAT_01CBE5A4 (6 bytes: mov eax,[01cbe5a4]; ret). |
| `01479f30` | CloneState::SetStaticValue | likely | Virtual slot [13] of CloneState. Takes one stack argument and stores it into the global DAT_01CBE5A4; ret 4 (12 bytes). |
| `0147a5d0` | CloneState::DeserializeIds | likely | Virtual slot [9] (+0x24) of CloneState. Calls this's vtable slot +0x10 with (param_2, &param_2) and requires a nonzero byte; then 0147c850(param_2) must be non… |

**CollectionItem** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013d4720` | CollectionItem::~CollectionItem | confirmed | Destructor body: sets the vtable to CollectionItem::vftable, calls member destructor 012c2de0 on the objects at this+0x20 and this+0x10, then resets the vtable… |
| `013d47c0` | CollectionItem::TypeName | confirmed | Virtual slot [11] of CollectionItem: returns the address of the string "CollectionItem" (mov eax, 018e1488; ret). |
| `013d47d0` | CollectionItem::GetStaticDword | confirmed | Virtual slot [12] of CollectionItem: returns the global dword at 01cbde20 (mov eax,[01cbde20]; ret). |
| `013d47e0` | CollectionItem::SetStaticDword | confirmed | Virtual slot [13] of CollectionItem: stores its stack argument into the global dword 01cbde20 (mov eax,[esp+4]; mov [01cbde20],eax; ret 4). |
| `013d4c30` | CollectionItem::DeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): calls the destructor body 013d4720, then frees the object with _free only when bit 0 of the second argument is se… |
| `013d55b0` | CollectionItem::CollectionItem | confirmed | Default constructor: stores the PersistentBase, PersistentComponent and CollectionItem vtables in that order, zeroes the ArchetypeID (+4) and Quantity desired … |

**CommandObject_AddAttrModFilter** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0150fb30` | CommandObject_AddAttrModFilter::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape: calls the destructor body 0150f7d0, tests bit 0 of the stack argument and, when set, frees this with _free (00d82463); return… |
| `0150fb50` | CommandObject_AddAttrModFilter::CommandObject_AddAttrModFil… | confirmed | Default constructor (fastcall): calls base ctor 0142b070, stores vftables 0191091c at offset 0 and 019108e0 at +8, calls 012fa910 (member init), assigns the 30… |
| `0150f7d0` | CommandObject_AddAttrModFilter::~CommandObject_AddAttrModFi… | likely | Stores vftables 0191091c at offset 0 and 019108e0 at +8, calls 01300cd0 (member cleanup), then calls 0142af20 (CommandObject base destructor body). Returns not… |
| `0150f8a0` | CommandObject_AddAttrModFilter::doCommand | likely | Slot [17] of the primary AddAttrModFilter vftable (0191091c). Gets the target from 01381460 and casts it to Card (00d8d382); returns 0 when the cast fails. Oth… |
| `0150f990` | CommandObject_AddAttrModFilter::undoCommand | likely | Slot [19] of the primary AddAttrModFilter vftable (0191091c). Gets the target from 01381460, casts it to Card (00d8d382) and returns 0 on failure. Otherwise ca… |
| `0150fb20` | CommandObject_AddAttrModFilter::DeletingDtorThunk | likely | Thunk: calls 0150fb30 (secondary-entry thunk to the scalar deleting destructor; 8 bytes, the same subtract-8 and jump shape as the other command thunks). |

**CommandObject_AddGameText** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0150f580` | CommandObject_AddGameText::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape: calls the destructor body 0150f3f0, tests bit 0 of the stack argument and, when set, frees this with _free (00d82463); return… |
| `0150f640` | CommandObject_AddGameText::CommandObject_AddGameText | confirmed | Default constructor (fastcall): calls base ctor 0142b070, stores vftables 019107ac at offset 0 and 01910770 at +8, and assigns the 25-char std::string 'Command… |
| `0150f6b0` | CommandObject_AddGameText::CommandObject_AddGameText | confirmed | Constructor overload with one stack argument: calls base ctor 0142b150 with it, stores vftables, assigns the name, and stores 013813e0's result at +0x64. |
| `0150f3f0` | CommandObject_AddGameText::~CommandObject_AddGameText | likely | Stores vftables 019107ac at offset 0 and 01910770 at +8, then calls 0142af20 (CommandObject base destructor body). Returns nothing. |
| `0150f4c0` | CommandObject_AddGameText::Load | likely | Secondary-vtable slot [9]. Reads through vcall +0x10 with the argument (assert 'false' at line 0x32 in CommandObjectAddGameText.cpp), calls 0142a880 (assert at… |
| `0150f520` | CommandObject_AddGameText::Save | likely | Secondary-vtable slot [10]. Calls vcall +8 of the object, writes 'Starting CommandObject_AddGameText' through 013c26e0 (no-op), calls 0142acd0, writes 'Ending … |
| `0150f570` | CommandObject_AddGameText::DeletingDtorThunk | likely | Thunk: calls 0150f580 (secondary-entry thunk to the scalar deleting destructor; 8 bytes, the same subtract-8 and jump shape as the other command thunks). |

**CommandObject_AddKeyword** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0150f1f0` | CommandObject_AddKeyword::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape: calls the destructor body 0150eef0, tests bit 0 of the stack argument and, when set, frees this with _free (00d82463); return… |
| `0150f210` | CommandObject_AddKeyword::CommandObject_AddKeyword | confirmed | Default constructor (fastcall): calls base ctor 0142b070, stores vftables 019105f4 at offset 0 and 019105b4 at +8, calls 012fa910 (member init) and assigns the… |
| `0150f290` | CommandObject_AddKeyword::CommandObject_AddKeyword | confirmed | Constructor overload with a flag byte and two more stack arguments (last one to the base ctor): calls base ctor 0142b150, stores vftables, calls 012fa910, assi… |
| `0150eef0` | CommandObject_AddKeyword::~CommandObject_AddKeyword | likely | Stores vftables 019105f4 at offset 0 and 019105b4 at +8, calls 01300cd0 (member cleanup), then calls 0142af20 (CommandObject base destructor body). Returns not… |
| `0150f1e0` | CommandObject_AddKeyword::DeletingDtorThunk | likely | Thunk: calls 0150f1f0 (secondary-entry thunk to the scalar deleting destructor; 8 bytes, the same subtract-8 and jump shape as the other command thunks). |

**CommandObject_AddSuppressEffect** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0150ecf0` | CommandObject_AddSuppressEffect::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape: calls the destructor body 0150eaa0, tests bit 0 of the stack argument and, when set, frees this with _free (00d82463); return… |
| `0150ed10` | CommandObject_AddSuppressEffect::CommandObject_AddSuppressE… | confirmed | Default constructor (fastcall): calls base ctor 0142b070, stores vftables 0191048c at offset 0 and 0191044c at +8, calls 012fa910 (member init) and assigns the… |
| `0150ed90` | CommandObject_AddSuppressEffect::CommandObject_AddSuppressE… | confirmed | Constructor overload with a flag byte and two more stack arguments (last one to the base ctor): calls base ctor 0142b150, stores vftables, calls 012fa910, assi… |
| `0150eaa0` | CommandObject_AddSuppressEffect::~CommandObject_AddSuppress… | likely | Stores vftables 0191048c at offset 0 and 0191044c at +8, calls 01300cd0 (member cleanup), then calls 0142af20 (CommandObject base destructor body). Returns not… |
| `0150eb70` | CommandObject_AddSuppressEffect::doCommand | likely | Slot [17] of the primary AddSuppressEffect vftable (0191048c). Asserts 'mTarget' at line 0x25 in CommandObjectAddSuppressEffect.cpp when +0x68 is null, calls 0… |
| `0150ebb0` | CommandObject_AddSuppressEffect::undoCommand | likely | Slot [19] of the primary AddSuppressEffect vftable (0191048c). Asserts 'mTarget' at line 0x2f in CommandObjectAddSuppressEffect.cpp when +0x68 is null, calls 0… |
| `0150ece0` | CommandObject_AddSuppressEffect::DeletingDtorThunk | likely | Thunk: subtracts 8 from ECX and jumps to 0150ecf0 (bytes 83 e9 08; e9 08 00 00 00). |

**CommandObject_AddToken** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0150e300` | CommandObject_AddToken::doCommand | confirmed | Slot [17] of the primary AddToken vftable (019102f4). Asserts 'mTarget' at line 0x5c when +0x68 is null, casts +0x68 to Card (00d8d382), logs 'CommandObject_Ad… |
| `0150e750` | CommandObject_AddToken::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape: calls the destructor body 0150e1f0, tests bit 0 of the stack argument and, when set, frees this with _free (00d82463); return… |
| `0150e770` | CommandObject_AddToken::CommandObject_AddToken | confirmed | Default constructor (fastcall): calls base ctor 0142b070, stores vftables 019102f4 at offset 0 and 019102b4 at +8, calls 012fa910 (member init), assigns the 22… |
| `0150e810` | CommandObject_AddToken::CommandObject_AddToken | confirmed | Constructor overload with a flag byte and three more stack arguments (last one to the base ctor): calls base ctor 0142b150, stores vftables, calls 012fa910, as… |
| `0150e910` | CommandObject_AddToken::CommandObject_AddToken | confirmed | Constructor overload with a flag byte and three more stack arguments (last one to the base ctor): calls base ctor 0142b150, stores vftables, calls 012fa910, as… |
| `0150e1f0` | CommandObject_AddToken::~CommandObject_AddToken | likely | Stores the AddToken vftables 019102f4 at offset 0 and 019102b4 at +8, calls 01300cd0 (member cleanup in the same EH-state pattern), then calls 0142af20 (Comman… |
| `0150e740` | CommandObject_AddToken::DeletingDtorThunk | likely | Thunk: subtracts 8 from ECX and jumps to 0150e750 (bytes 83 e9 08; e9 08 00 00 00). |

**CommandObject_AlternateMultiAction** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0150df30` | CommandObject_AlternateMultiAction::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape: calls the destructor body 0150dda0, tests bit 0 of the stack argument and, when set, frees this with _free (00d82463); return… |
| `0150df50` | CommandObject_AlternateMultiAction::CommandObject_Alternate… | confirmed | Default constructor (fastcall): calls base ctor 0142b070, stores vftables 0191017c at offset 0 and 01910140 at +8, assigns the 34-char std::string 'CommandObje… |
| `0150dfe0` | CommandObject_AlternateMultiAction::CommandObject_Alternate… | confirmed | Constructor overload with five stack arguments (the last one goes to the base ctor): calls base ctor 0142b150, stores vftables, assigns the name, sets byte +0x… |
| `0150dda0` | CommandObject_AlternateMultiAction::~CommandObject_Alternat… | likely | Stores vftables 0191017c at offset 0 and 01910140 at +8, then calls 0142af20 (CommandObject base destructor body). Returns nothing. |
| `0150de70` | CommandObject_AlternateMultiAction::Load | likely | Secondary-vtable slot [9]. Reads through vcall +0x10 of the object with the argument, asserts 'false' at line 0x4f in CommandObjectAlternateMultiAction.cpp whe… |
| `0150ded0` | CommandObject_AlternateMultiAction::Save | likely | Secondary-vtable slot [10]. Calls vcall +8 of the object (slot 2), writes the trace text 'Starting CommandObject_AlternateMultiAction' through 013c26e0 (no-op)… |
| `0150df20` | CommandObject_AlternateMultiAction::DeletingDtorThunk | likely | Thunk: subtracts 8 from ECX and jumps to 0150df30 (bytes 83 e9 08; e9 08 00 00 00). |

**CommandObject_Attach** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0150db80` | CommandObject_Attach::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape: calls the destructor body 0150d9d0, tests bit 0 of the stack argument and, when set, frees this with _free (00d82463); return… |
| `0150dba0` | CommandObject_Attach::CommandObject_Attach | confirmed | Default constructor (fastcall): calls base ctor 0142b070, stores vftables 0191003c at offset 0 and 01910000 at +8, and assigns the 20-char std::string 'Command… |
| `0150dc10` | CommandObject_Attach::CommandObject_Attach | confirmed | Constructor overload with one stack argument: calls base ctor 0142b150 with it, stores vftables, assigns the name, stores 01381460's result as the target at +0… |
| `0150d9d0` | CommandObject_Attach::~CommandObject_Attach | likely | Stores vftables 0191003c at offset 0 and 01910000 at +8, then calls 0142af20 (CommandObject base destructor body). Returns nothing. |
| `0150dac0` | CommandObject_Attach::Load | likely | Secondary-vtable slot [9]. Reads through vcall +0x10 of the object with the argument, asserts 'false' at line 99 in CommandObjectAttach.cpp when it fails, call… |
| `0150db20` | CommandObject_Attach::Save | likely | Secondary-vtable slot [10]. Calls vcall +8 of the object (slot 2), writes the trace text 'Starting CommandObject_Attach' through 013c26e0 (no-op), calls 0142ac… |
| `0150db70` | CommandObject_Attach::DeletingDtorThunk | likely | Thunk: subtracts 8 from ECX and jumps to 0150db80 (bytes 83 e9 08; e9 08 00 00 00). |

**CommandObject_CancelCommand** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0150d800` | CommandObject_CancelCommand::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape: calls the destructor body 0150d660, tests bit 0 of the stack argument and, when set, frees this with _free (00d82463); return… |
| `0150d820` | CommandObject_CancelCommand::CommandObject_CancelCommand | confirmed | Default constructor (fastcall): calls base ctor 0142b070, stores vftables 0190fecc and 0190fe8c at +8, assigns the 27-char name 'CommandObject_CancelCommand' t… |
| `0150d8a0` | CommandObject_CancelCommand::CommandObject_CancelCommand | confirmed | Constructor overload with one stack argument: calls base ctor 0142b150 with it, stores vftables, assigns the name, stores 01381380's result at +0x30 and 013813… |
| `0150d660` | CommandObject_CancelCommand::~CommandObject_CancelCommand | likely | Stores vftables 0190fecc at offset 0 and 0190fe8c at +8, then calls 0142af20 (CommandObject base destructor body). Returns nothing. |
| `0150d710` | CommandObject_CancelCommand::doCommand | likely | Slot [17] of the primary CancelCommand vftable (0190fecc). Calls 013508b0 (a logger that emits text only when its flags are set) with this+0x30 and the text 'C… |
| `0150d740` | CommandObject_CancelCommand::Load | likely | Secondary-vtable slot [9]. Reads through vcall +0x10 of the object with the argument, asserts 'false' at line 0x32 in CommandObjectCancelCommand.cpp when it fa… |
| `0150d7a0` | CommandObject_CancelCommand::Save | likely | Secondary-vtable slot [10]. Calls vcall +8 of the object (slot 2), writes the trace text 'Starting CommandObject_CancelCommand' through 013c26e0 (no-op), calls… |
| `0150d7f0` | CommandObject_CancelCommand::DeletingDtorThunk | likely | Thunk: calls 0150d800 (the deleting destructor body with this unchanged; bytes 8 long, 83 e9 08 e9 08 00 00 00 read as sub ecx,8; jmp). |

**CommandObject_ChainedAction** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0150d3b0` | CommandObject_ChainedAction::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape: calls the destructor body 0150d1a0, tests bit 0 of the stack argument and, when set, frees this with _free (00d82463); return… |
| `0150d3d0` | CommandObject_ChainedAction::CommandObject_ChainedAction | confirmed | Default constructor (fastcall): calls base ctor 0142b070, stores vftables 0190fd74 and 0190fd38 at +8, assigns the 27-char name 'CommandObject_ChainedAction' t… |
| `0150d450` | CommandObject_ChainedAction::CommandObject_ChainedAction | confirmed | Constructor overload with two stack arguments: calls base ctor 0142b150 with the second argument, stores vftables, sets name, byte +0x118 = 1, dword +0x11c = 0… |
| `0150d1a0` | CommandObject_ChainedAction::~CommandObject_ChainedAction | likely | Stores vftables 0190fd74 at offset 0 and 0190fd38 at +8, then calls 0142af20 (CommandObject base destructor body). Returns nothing. |
| `0150d340` | CommandObject_ChainedAction::Save | likely | Secondary-vtable slot [10]. Calls vcall +8 of the object (slot 2), writes the trace text 'Starting CommandObject_ChainedAction' via 013c26e0 (no-op), calls 014… |
| `0150d3a0` | CommandObject_ChainedAction::DeletingDtorThunk | likely | Thunk: subtracts 8 from ECX and jumps to 0150d3b0 (bytes 83 e9 08; e9 08 00 00 00). |

**CommandObject_Clone** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0150cf50` | CommandObject_Clone::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape: calls the destructor body 0150cc70, tests bit 0 of the stack argument and, when set, frees this with _free (00d82463); return… |
| `0150cf70` | CommandObject_Clone::CommandObject_Clone | confirmed | Default constructor (fastcall): calls base ctor 0142b070, stores vftables 0190fc34 at offset 0 and 0190fbf8 at +8, assigns the 19-char std::string 'CommandObje… |
| `0150cff0` | CommandObject_Clone::CommandObject_Clone | confirmed | Constructor overload with one stack argument: calls base ctor 0142b150 with it, stores vftables 0190fc34 and 0190fbf8, assigns the 19-char name 'CommandObject_… |
| `0150cc70` | CommandObject_Clone::~CommandObject_Clone | likely | Stores vftables 0190fc34 at offset 0 and 0190fbf8 at +8, then calls 0142af20 (CommandObject base destructor body). Returns nothing. |
| `0150cd40` | CommandObject_Clone::undoCommand | likely | Slot [19] of the primary CommandObject_Clone vftable (0190fc34). Looks up the card id at +0x124 with 01357a00 and dynamic-casts it to Card (00d8d382). When non… |
| `0150ce50` | CommandObject_Clone::Load | likely | Secondary-vtable slot [9]. Reads through vcall +0x10 of the object with the argument, asserts 'false' at line 0x61 in CommandObjectClone.cpp when it fails, cal… |
| `0150cee0` | CommandObject_Clone::Save | likely | Secondary-vtable slot [10]. Calls vcall +8 of the object (slot 2), writes the trace text 'Starting CommandObject_Clone' via 013c26e0 (no-op), calls 0142acd0, w… |
| `0150cf40` | CommandObject_Clone::DeletingDtorThunk | likely | Thunk: subtracts 8 from ECX and jumps to 0150cf50 (bytes 83 e9 08; e9 08 00 00 00). |

**CommandObject_CreateCard** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0150c280` | CommandObject_CreateCard::doCommand | confirmed | Slot [17] of the primary CreateCard vftable. Logs 'CommandObject_CreateCard::doCommand %d' with +0x128 and looks up the object with 01357a00; returns 0 when th… |
| `0150c6c0` | CommandObject_CreateCard::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape: calls the destructor body 0150c1c0, tests bit 0 of the stack argument and, when set, frees this with _free (00d82463); return… |
| `0150c780` | CommandObject_CreateCard::CommandObject_CreateCard | confirmed | Default constructor (fastcall): calls base ctor 0142b070, stores vftables 0190f9f4 at offset 0 and 0190f9b8 at +8, zeroes dwords +0x124 and +0x128, sets byte +… |
| `0150c1c0` | CommandObject_CreateCard::~CommandObject_CreateCard | likely | Stores vftables 0190f9f4 at offset 0 and 0190f9b8 at +8, then calls 0142af20 (CommandObject base destructor body). Returns nothing. |
| `0150c4e0` | CommandObject_CreateCard::Load | likely | Secondary-vtable slot [9]. Reads through vcall +0x10 of the object with a local out-value, asserts 'false' at line 0xaf in CommandObjectCreateCard.cpp when it … |
| `0150c610` | CommandObject_CreateCard::Save | likely | Secondary-vtable slot [10]. Calls vcall +8 of the object (slot 2), writes the trace text Starting CommandObject_CreateCard through 013c26e0 (no-op), calls 0142… |
| `0150c6b0` | CommandObject_CreateCard::DeletingDtorThunk | likely | Thunk: subtracts 8 from ECX and jumps to 0150c6c0 (bytes 83 e9 08; e9 08 00 00 00). |

**CommandObject_CustomChoiceDialog** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0150bdd0` | CommandObject_CustomChoiceDialog::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape: calls the destructor body 0150bc20, tests bit 0 of the stack argument and, when set, frees this with _free (00d82463, thunk t… |
| `0150bdf0` | CommandObject_CustomChoiceDialog::CommandObject_CustomChoic… | confirmed | Default constructor (fastcall, no stack arguments): calls base ctor 0142b070, stores vftables 0190f884 at offset 0 and 0190f844 at +8, assigns the 32-char std:… |
| `0150be70` | CommandObject_CustomChoiceDialog::CommandObject_CustomChoic… | confirmed | Constructor overload with five stack arguments (plus a sixth used only as base argument): calls base ctor 0142b150 with the last argument, stores the vftables,… |
| `0150bfe0` | CommandObject_CustomChoiceDialog::CommandObject_CustomChoic… | confirmed | Constructor overload with six stack arguments: calls base ctor 0142b150 with the last argument, stores the vftables, sets name, byte +0x118 = 1, dword +0x11c =… |
| `0150bc20` | CommandObject_CustomChoiceDialog::~CommandObject_CustomChoi… | likely | Stores vftables 0190f884 at offset 0 and 0190f844 at +8, then calls 0142af20 (CommandObject base destructor body, referenced by CommandObject::vftable and call… |
| `0150bd10` | CommandObject_CustomChoiceDialog::Load | likely | Secondary-vtable slot [9]. Calls vcall +0x10 of the object with the argument and &argument (a read), asserts 'false' at line 0x6b in CommandObjectCustomChoiceD… |
| `0150bd70` | CommandObject_CustomChoiceDialog::Save | likely | Secondary-vtable slot [10]. Calls vcall +8 of the object (slot 2) with the argument, calls 013c26e0 (no-op returning 1) with the text Starting CommandObject_Cu… |
| `0150bdc0` | CommandObject_CustomChoiceDialog::DeletingDtorThunk | likely | Thunk: subtracts 8 from ECX and jumps to 0150bdd0 (bytes 83 e9 08; e9 08 00 00 00). Passes the stack argument through unchanged. |

**CommandObject_Discard** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0150adc0` | CommandObject_Discard::Write | confirmed | Writes the Discard body: calls slot [2] of this, no-op trace Starting text, base body 0142acd0, writes the int at +0x11c (Discard Reason), writes the id from 0… |
| `0150aea0` | CommandObject_Discard::Dtor | confirmed | Destructor body: stores the Discard vftables at +0 and +8; walks the 4-byte element vector from +0x130 to +0x134, for each id resolves the object with 01357a00… |
| `0150afe0` | CommandObject_Discard::GetTypeName | confirmed | Returns the address of the literal CommandObject_Discard (mov eax,imm32; ret). |
| `0150b030` | CommandObject_Discard::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls destructor body 0150aea0, frees the block with _free when bit 0 of the flag argument is set, and returns this. |
| `0150b2e0` | CommandObject_Discard::CommandObject_Discard | confirmed | Calls 0142b070 (CommandObject base init), stores vftable 0190f614 at offset 0 and 0190f5d4 at +8, zeroes dwords +0x130, +0x134, +0x138, assigns the 21-char std… |
| `0150baf0` | CommandObject_Discard::CommandObject_Discard | confirmed | Constructor overload with two stack arguments (ret 8): stores vftables 0190f614 at offset 0 and 0190f5d4 at +8, calls base ctor 0142b150 with the first argumen… |
| `0150bb80` | CommandObject_Discard::CommandObject_Discard | confirmed | Constructor overload with three stack arguments (ret 12): stores vftables 0190f614 and 0190f5d4 at +8, calls base ctor 0142b150 with the first argument, zeroes… |
| `0150aff0` | CommandObject_Discard::GetGlobalDword | likely | Returns the dword global DAT_01cbea38 (mov eax,[global]; ret). |
| `0150b000` | CommandObject_Discard::SetGlobalDword | likely | Stores its stack argument to the dword global DAT_01cbea38 and returns with ret 4. |
| `0150b020` | CommandObject_Discard::ScalarDeletingDtorThunk | likely | Secondary-vtable thunk: subtracts 8 from ECX (sub ecx,8) and jumps to 0150b030. |
| `0150b430` | CommandObject_Discard::InitDiscard | likely | Sets the name field (+0x48) to 'CommandObject_Discard', calls 01381460 and stores the result as the target at +0x68, and checks it with vcall +0x30. Reads the … |
| `0150b960` | CommandObject_Discard::Load | likely | Secondary-vtable slot [9] (0190f5d4 +0x8). Reads values through vcall +0x10 of the object and calls 0142a880, 013c24d0 (three reads, each checked and asserted … |

**CommandObject_DiscardPile** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0150a6c0` | CommandObject_DiscardPile::Dtor | confirmed | Destructor body: stores the DiscardPile vftables at +0 and +8, then calls the base destructor 0142af20. No owned member is released here. |
| `0150a730` | CommandObject_DiscardPile::GetTypeName | confirmed | Returns the address of the literal CommandObject_DiscardPile (mov eax,imm32; ret). |
| `0150a7a0` | CommandObject_DiscardPile::Read | confirmed | Reads the DiscardPile body: calls slot [4] of this with the stream and a version output, checks the stream with 0142a880 (assert at line 0xbf), then reads an i… |
| `0150a900` | CommandObject_DiscardPile::Write | confirmed | Writes the DiscardPile body: calls slot [2] of this, no-op trace Starting text, base body 0142acd0, then writes the ints at +0x11c (DiscardReason), +0x120 (Num… |
| `0150a9f0` | CommandObject_DiscardPile::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls destructor body 0150a6c0, frees the block with _free when bit 0 of the flag argument is set, and returns this. |
| `0150aa10` | CommandObject_DiscardPile::Ctor | confirmed | Default constructor: calls base ctor 0142b070, stores the DiscardPile vftables, assigns the name string CommandObject_DiscardPile, zeroes byte +0x131, dwords +… |
| `0150aab0` | CommandObject_DiscardPile::CtorWithArgs | confirmed | Constructor overload with a base argument and five values: calls base ctor 0142b150, stores the DiscardPile vftables, assigns the name string, asserts 013813e0… |
| `0150a740` | CommandObject_DiscardPile::GetGlobalDword | likely | Returns the dword global DAT_01cbea34 (mov eax,[global]; ret). |
| `0150a750` | CommandObject_DiscardPile::SetGlobalDword | likely | Stores its stack argument to the dword global DAT_01cbea34 and returns with ret 4. |
| `0150a9e0` | CommandObject_DiscardPile::ScalarDeletingDtorThunk | likely | Secondary-vtable thunk: subtracts 8 from ECX (sub ecx,8) and jumps to 0150a9f0. |

**CommandObject_DrawCard** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01509f10` | CommandObject_DrawCard::Dtor | confirmed | Destructor body: stores the DrawCard vftables at +0 and +8, then calls the base destructor 0142af20. |
| `01509f80` | CommandObject_DrawCard::GetTypeName | confirmed | Returns the address of the literal CommandObject_DrawCard (mov eax,imm32; ret). |
| `0150a0b0` | CommandObject_DrawCard::Read | confirmed | Reads the DrawCard body: calls slot [4] of this with the stream and a version output, checks the stream with 0142a880, then reads four integers into +0x11c, +0… |
| `0150a270` | CommandObject_DrawCard::Write | confirmed | Writes the DrawCard body: calls slot [2] of this, no-op trace, base body 0142acd0, writes the ints at +0x11c (CardSource), +0x120 (Card Draw Position), +0x124 … |
| `0150a380` | CommandObject_DrawCard::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls destructor body 01509f10, frees the block with _free when bit 0 of the flag argument is set, and returns this. |
| `0150a3a0` | CommandObject_DrawCard::Ctor | confirmed | Default constructor: calls base ctor 0142b070, stores the DrawCard vftables, assigns the name string CommandObject_DrawCard to +0x48, sets +0x124 = 0, +0x128 =… |
| `0150a450` | CommandObject_DrawCard::CtorWithDraw | confirmed | Constructor overload taking five values and a base argument: calls base ctor 0142b150 with the last argument, stores the DrawCard vftables, assigns the name st… |
| `01509f90` | CommandObject_DrawCard::GetGlobalDword | likely | Returns the dword global DAT_01cbea30 (mov eax,[global]; ret). |
| `01509fa0` | CommandObject_DrawCard::SetGlobalDword | likely | Stores its stack argument to the dword global DAT_01cbea30 and returns with ret 4. |
| `0150a370` | CommandObject_DrawCard::ScalarDeletingDtorThunk | likely | Secondary-vtable thunk: subtracts 8 from ECX (sub ecx,8) and jumps to 0150a380. |

**CommandObject_DummyEvent** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01509b90` | CommandObject_DummyEvent::Dtor | confirmed | Destructor body: stores the DummyEvent vftables at +0 and +8, then calls the base destructor 0142af20. No owned member is released here. |
| `01509c00` | CommandObject_DummyEvent::GetTypeName | confirmed | Returns the address of the literal CommandObject_DummyEvent (mov eax,imm32; ret). |
| `01509c60` | CommandObject_DummyEvent::Read | confirmed | Reads the DummyEvent body: calls slot [4] of this with the stream (assert false at line 0x2e on failure), checks the stream with 0142a880 (assert at 0x31), the… |
| `01509cc0` | CommandObject_DummyEvent::Write | confirmed | Writes the DummyEvent body: calls slot [2] of this, no-op trace Starting text, base body 0142acd0, no-op trace Ending text, then calls slot [3] of this. |
| `01509d20` | CommandObject_DummyEvent::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls destructor body 01509b90, frees the block with _free when bit 0 of the flag argument is set, and returns this. |
| `01509d40` | CommandObject_DummyEvent::Ctor | confirmed | Default constructor: calls base ctor 0142b070, stores the DummyEvent vftables at +0 and +8, assigns the name string CommandObject_DummyEvent to +0x48, zeroes t… |
| `01509dc0` | CommandObject_DummyEvent::CtorWithObject | confirmed | Constructor overload taking an object (param_2): calls base ctor 0142b150 with it, stores the DummyEvent vftables, assigns the name string, then with ECX = par… |
| `01509c10` | CommandObject_DummyEvent::GetGlobalDword | likely | Returns the dword global DAT_01cbea2c (mov eax,[global]; ret). |
| `01509c20` | CommandObject_DummyEvent::SetGlobalDword | likely | Stores its stack argument to the dword global DAT_01cbea2c and returns with ret 4. |
| `01509d10` | CommandObject_DummyEvent::ScalarDeletingDtorThunk | likely | Secondary-vtable thunk: subtracts 8 from ECX (sub ecx,8) and jumps to 01509d20. |

**CommandObject_EmitSound** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `015096d0` | CommandObject_EmitSound::Dtor | confirmed | Destructor body: stores the EmitSound vftables at +0 and +8, then calls the base destructor 0142af20. |
| `01509740` | CommandObject_EmitSound::GetTypeName | confirmed | Returns the address of the literal CommandObject_EmitSound (mov eax,imm32; ret). |
| `01509780` | CommandObject_EmitSound::DoCommand | confirmed | Traces the doCommand text with the value at +0x124 through 013508b0 (format CommandObject_EmitSound::doCommand %d) on this+0x30, allocates 0x14 bytes (00d833a6… |
| `01509850` | CommandObject_EmitSound::Read | confirmed | Reads the EmitSound body: calls slot [4] of this with the stream, checks the stream with 0142a880 (assert false at line 0x42 on failure), reads one int with 01… |
| `015098e0` | CommandObject_EmitSound::Write | confirmed | Writes the EmitSound body: calls slot [2] of this, no-op trace Starting text, base body 0142acd0, writes the int at +0x11c with 013c23f0, no-op trace Ending te… |
| `01509950` | CommandObject_EmitSound::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls destructor body 015096d0, frees the block with _free when bit 0 of the flag argument is set, and returns this. |
| `01509970` | CommandObject_EmitSound::Ctor | confirmed | Default constructor: calls base ctor 0142b070, stores the EmitSound vftables at +0 and +8, zeroes the dword at +0x124 and assigns the name string CommandObject… |
| `015099e0` | CommandObject_EmitSound::CtorWithSoundValue | confirmed | Constructor overload taking a value (param_2) and a base argument: calls base ctor 0142b150 with the third argument, stores the EmitSound vftables, zeroes +0x1… |
| `01509750` | CommandObject_EmitSound::GetGlobalDword | likely | Returns the dword global DAT_01cbea28 (mov eax,[global]; ret). |
| `01509760` | CommandObject_EmitSound::SetGlobalDword | likely | Stores its stack argument to the dword global DAT_01cbea28 and returns with ret 4. |
| `01509940` | CommandObject_EmitSound::ScalarDeletingDtorThunk | likely | Secondary-vtable thunk: subtracts 8 from ECX (sub ecx,8) and jumps to 01509950. |

**CommandObject_EmitTextMessage** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0141f340` | CommandObject_EmitTextMessage::Ctor | confirmed | Thiscall constructor: stores the CommandObject_EmitTextMessage vtables at +0 and +8, assigns the 29-char name to the string at +0x48, copies the std::string ar… |
| `0141ffc0` | CommandObject_EmitTextMessage::doCommand | confirmed | Virtual slot 17 of CommandObject_EmitTextMessage::vftable. Traces 'EmitTextMessage %s' with the text at +0x128, returns early when 01383a10 is set, then switch… |
| `01423300` | CommandObject_EmitTextMessage::ReadFromSerial | confirmed | Virtual slot 9 of the secondary vtable CommandObject_EmitTextMessage::vftable (018e6b9c). Calls vtable +0x10 with param_2 and a local, checks 0142a880(param_2)… |

**CommandObject_GameProperties** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01509240` | CommandObject_GameProperties::GetTypeName | confirmed | Returns the address of the literal CommandObject_GameProperties (mov eax,imm32; ret). |
| `015092d0` | CommandObject_GameProperties::Read | confirmed | Reads the GameProperties body: calls slot [4] of this with the stream, checks the stream with 0142a880 (assert false at line 0x46 or 0x49 on failure), reads an… |
| `01509400` | CommandObject_GameProperties::Write | confirmed | Writes the GameProperties body: calls slot [2] of this, no-op trace Starting text, base body 0142acd0, trace Property with the int at +0x11c (013c23f0), trace … |
| `015094c0` | CommandObject_GameProperties::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls destructor body 01509190, frees the block with _free when bit 0 of the flag argument is set, and returns this. |
| `01509570` | CommandObject_GameProperties::CtorWithArgs | confirmed | Constructor overload: calls base ctor 0142b150 with its fourth argument, stores the GameProperties vftables, calls 012fa910, assigns the name string, stores th… |
| `01509250` | CommandObject_GameProperties::GetGlobalDword | likely | Returns the dword global DAT_01cbea24 (mov eax,[global]; ret). |
| `01509260` | CommandObject_GameProperties::SetGlobalDword | likely | Stores its stack argument to the dword global DAT_01cbea24 and returns with ret 4. |
| `015094b0` | CommandObject_GameProperties::ScalarDeletingDtorThunk | likely | Secondary-vtable thunk: subtracts 8 from ECX (sub ecx,8) and jumps to 015094c0. |

**CommandObject_GetTarget** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01508d30` | CommandObject_GetTarget::Dtor | confirmed | Destructor body: stores the GetTarget vftables at +0 and +8, then calls the base destructor 0142af20. |
| `01508da0` | CommandObject_GetTarget::GetTypeName | confirmed | Returns the address of the literal CommandObject_GetTarget (mov eax,imm32; ret). |
| `01508e00` | CommandObject_GetTarget::Read | confirmed | Reads the GetTarget body: calls slot [4] of this with the stream and a parameter slot, checks the stream with 0142a880 (assert false at line 0x5d or 0x60 on fa… |
| `01508e60` | CommandObject_GetTarget::Write | confirmed | Writes the GetTarget body: calls slot [2] of this, no-op trace Starting text, base body 0142acd0, no-op trace Ending text, then calls slot [3] of this. |
| `01508ec0` | CommandObject_GetTarget::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls destructor body 01508d30, frees the block with _free when bit 0 of the flag argument is set, and returns this. |
| `01508ee0` | CommandObject_GetTarget::Ctor | confirmed | Default constructor: calls base ctor 0142b070, stores the GetTarget vftables at +0 and +8, assigns the name string CommandObject_GetTarget to +0x48, zeroes byt… |
| `01508f60` | CommandObject_GetTarget::CtorWithState | confirmed | Constructor overload taking a base argument and four more values: calls base ctor 0142b150 with its last argument, stores the GetTarget vftables, assigns the n… |
| `01508db0` | CommandObject_GetTarget::GetGlobalDword | likely | Returns the dword global DAT_01cbea20 (mov eax,[global]; ret). |
| `01508dc0` | CommandObject_GetTarget::SetGlobalDword | likely | Stores its stack argument to the dword global DAT_01cbea20 and returns with ret 4. |
| `01508eb0` | CommandObject_GetTarget::ScalarDeletingDtorThunk | likely | Secondary-vtable thunk: subtracts 8 from ECX (sub ecx,8) and jumps to 01508ec0. |

**CommandObject_GetTopCard** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01508950` | CommandObject_GetTopCard::Dtor | confirmed | Destructor body: stores the GetTopCard vftables at +0 and +8, then calls the base destructor 0142af20. |
| `015089c0` | CommandObject_GetTopCard::GetTypeName | confirmed | Returns the address of the literal CommandObject_GetTopCard (mov eax,imm32; ret). |
| `01508a00` | CommandObject_GetTopCard::DoCommand | confirmed | Calls 013508b0 with this+0x30 and the literal CommandObject_GetTopCard::doCommand, then returns 1. No game state is changed in this body. |
| `01508a30` | CommandObject_GetTopCard::Read | confirmed | Reads the GetTopCard body: calls slot [4] of this with the stream and a parameter slot, checks the stream with 0142a880 (assert false at line 0x35 or 0x38 on f… |
| `01508a90` | CommandObject_GetTopCard::Write | confirmed | Writes the GetTopCard body: calls slot [2] of this, no-op trace Starting text, base body 0142acd0, no-op trace Ending text, then calls slot [3] of this. |
| `01508af0` | CommandObject_GetTopCard::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls destructor body 01508950, frees the block with _free when bit 0 of the flag argument is set, and returns this. |
| `01508b10` | CommandObject_GetTopCard::Ctor | confirmed | Default constructor: calls base ctor 0142b070, stores the GetTopCard vftables at +0 and +8, assigns the name string CommandObject_GetTopCard to +0x48, and zero… |
| `01508b90` | CommandObject_GetTopCard::CtorWithState | confirmed | Constructor overload taking a base argument and three values: calls base ctor 0142b150, stores the GetTopCard vftables, assigns the name string, sets +0x30 fro… |
| `015089d0` | CommandObject_GetTopCard::GetGlobalDword | likely | Returns the dword global DAT_01cbea1c (mov eax,[global]; ret). |
| `015089e0` | CommandObject_GetTopCard::SetGlobalDword | likely | Stores its stack argument to the dword global DAT_01cbea1c and returns with ret 4. |
| `01508ae0` | CommandObject_GetTopCard::ScalarDeletingDtorThunk | likely | Secondary-vtable thunk: subtracts 8 from ECX (sub ecx,8) and jumps to 01508af0. |

**CommandObject_InfoBox** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01507b50` | CommandObject_InfoBox::GetTypeName | confirmed | Returns the address of the literal CommandObject_InfoBox (mov eax,imm32; ret). |
| `01507b90` | CommandObject_InfoBox::Read | confirmed | Reads the InfoBox body: calls slot [4] of this with the stream, checks the stream with 0142a880 (assert false at line 0xb9 on failure), calls slot [9] (Read) o… |
| `01507c30` | CommandObject_InfoBox::Write | confirmed | Writes the InfoBox body: calls slot [2] of this, no-op trace Starting text, base body 0142acd0, slot [10] (Write) of the embedded sub-object at +0x11c, no-op t… |
| `01507ca0` | CommandObject_InfoBox::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls destructor body 01507ac0, frees the block with _free when bit 0 of the flag argument is set, and returns this. |
| `01507cc0` | CommandObject_InfoBox::Ctor | confirmed | Default constructor: calls base ctor 0142b070, stores the InfoBox vftables at +0 and +8, calls 012fa910, assigns the name string CommandObject_InfoBox to +0x48… |
| `01507d50` | CommandObject_InfoBox::CtorWithArg | confirmed | Constructor overload: calls base ctor 0142b150 with its third argument, stores the InfoBox vftables, calls 012fa910, assigns the name string, zeroes byte +0x81… |
| `01507df0` | CommandObject_InfoBox::DoCommand | confirmed | Traces CommandObject_InfoBox::doCommand through 013846b0 on this+0x84. Asserts the value at +0x128 is an IntValueMap (type id 0xe) and takes its map at +0x12c.… |
| `01507b60` | CommandObject_InfoBox::GetGlobalDword | likely | Returns the dword global DAT_01cbea18 (mov eax,[global]; ret). |
| `01507b70` | CommandObject_InfoBox::SetGlobalDword | likely | Stores its stack argument to the dword global DAT_01cbea18 and returns with ret 4. |
| `01507c90` | CommandObject_InfoBox::ScalarDeletingDtorThunk | likely | Secondary-vtable thunk: subtracts 8 from ECX (sub ecx,8) and jumps to 01507ca0. |

**CommandObject_InstallAction** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `015074e0` | CommandObject_InstallAction::Dtor | confirmed | Destructor body: stores the InstallAction vftables at +0 and +8, then calls the base destructor 0142af20. |
| `01507550` | CommandObject_InstallAction::GetTypeName | confirmed | Returns the address of the literal CommandObject_InstallAction (mov eax,imm32; ret). |
| `01507830` | CommandObject_InstallAction::Read | confirmed | Reads the InstallAction body: calls slot [4] of this with the stream, checks the stream with 0142a880, reads one int with 013c24d0 into +0x11c (assert false at… |
| `015078a0` | CommandObject_InstallAction::Write | confirmed | Writes the InstallAction body: calls slot [2] of this, no-op trace, base body 0142acd0, writes the int at +0x11c with 013c23f0, no-op trace Ending text, then c… |
| `01507910` | CommandObject_InstallAction::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls destructor body 015074e0, frees the block with _free when bit 0 of the flag argument is set, and returns this. |
| `01507930` | CommandObject_InstallAction::Ctor | confirmed | Default constructor: calls base ctor 0142b070, stores the InstallAction vftables at +0 and +8, zeroes the dword at +0x124, and assigns the name string CommandO… |
| `015079a0` | CommandObject_InstallAction::CtorWithOwner | confirmed | Constructor overload: calls base ctor 0142b150 with its third argument, stores the second argument at +0x124, stores the InstallAction vftables, and assigns th… |
| `01507560` | CommandObject_InstallAction::GetGlobalDword | likely | Returns the dword global DAT_01cbea14 (mov eax,[global]; ret). |
| `01507570` | CommandObject_InstallAction::SetGlobalDword | likely | Stores its stack argument to the dword global DAT_01cbea14 and returns with ret 4. |
| `01507900` | CommandObject_InstallAction::ScalarDeletingDtorThunk | likely | Secondary-vtable thunk: subtracts 8 from ECX (sub ecx,8) and jumps to 01507910. |

**CommandObject_InstallActionFilter** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01506f80` | CommandObject_InstallActionFilter::Dtor | confirmed | Destructor body: stores the InstallActionFilter vftables at +0 and +8, then calls the base destructor 0142af20. |
| `01506ff0` | CommandObject_InstallActionFilter::GetTypeName | confirmed | Returns the address of the literal CommandObject_InstallActionFilter (mov eax,imm32; ret). |
| `015070b0` | CommandObject_InstallActionFilter::Write | confirmed | Writes the InstallActionFilter body: calls slot [2] of this, no-op trace, base body 0142acd0, trace ValidActionFilter with 01340370 of +0x11c (0 when null) wri… |
| `01507160` | CommandObject_InstallActionFilter::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls destructor body 01506f80, frees the block with _free when bit 0 of the flag argument is set, and returns this. |
| `01507180` | CommandObject_InstallActionFilter::Read | confirmed | Reads the InstallActionFilter body: calls slot [4] of this with the stream and a local, checks the stream (0142a880, assert false at line 0x40), reads an int (… |
| `01507280` | CommandObject_InstallActionFilter::Ctor | confirmed | Default constructor: calls base ctor 0142b070, stores the InstallActionFilter vftables at +0 and +8, assigns the name string CommandObject_InstallActionFilter … |
| `01507300` | CommandObject_InstallActionFilter::CtorWithExpression | confirmed | Constructor overload taking an expression value (param_2) and an owner argument: calls base ctor 0142b150 with the owner, stores the InstallActionFilter vftabl… |
| `01507000` | CommandObject_InstallActionFilter::GetGlobalDword | likely | Returns the dword global DAT_01cbea10 (mov eax,[global]; ret). |
| `01507010` | CommandObject_InstallActionFilter::SetGlobalDword | likely | Stores its stack argument to the dword global DAT_01cbea10 and returns with ret 4. |
| `01507150` | CommandObject_InstallActionFilter::ScalarDeletingDtorThunk | likely | Secondary-vtable thunk: subtracts 8 from ECX (sub ecx,8) and jumps to 01507160. |

**CommandObject_InstallActionMap** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01506800` | CommandObject_InstallActionMap::GetTypeName | confirmed | Returns the address of the literal CommandObject_InstallActionMap (mov eax,imm32; ret). |
| `01506840` | CommandObject_InstallActionMap::GetDuration | confirmed | If this+0x138 (mAction) is non-zero, calls 013221a0, traces its value with format returning duration (013508b0 on this+0x30), calls 013221a0 again and returns … |
| `01506aa0` | CommandObject_InstallActionMap::Read | confirmed | Reads the InstallActionMap body: calls slot [4] of this with the stream, then slot [9] (Read) of the embedded sub-object at this+0x11c with the stream (the byt… |
| `01506c20` | CommandObject_InstallActionMap::Write | confirmed | Writes the InstallActionMap body: calls slot [2] of this, no-op trace, base body 0142acd0, trace ValueData with slot [10] of the sub-object at +0x11c, trace Ev… |
| `01506d20` | CommandObject_InstallActionMap::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls destructor body 01506770, frees the block with _free when bit 0 of the flag argument is set, and returns this. |
| `01506d40` | CommandObject_InstallActionMap::Ctor | confirmed | Default constructor: calls base ctor 0142b070, stores the InstallActionMap vftables at +0 and +8, calls 012fa910, assigns the name string CommandObject_Install… |
| `01506de0` | CommandObject_InstallActionMap::CtorWithArgs | confirmed | Constructor overload with seven parameters: calls base ctor 0142b150 with the last argument, stores the InstallActionMap vftables, calls 012fa910, assigns the … |
| `01506810` | CommandObject_InstallActionMap::GetGlobalDword | likely | Returns the dword global DAT_01cbea0c (mov eax,[global]; ret). |
| `01506820` | CommandObject_InstallActionMap::SetGlobalDword | likely | Stores its stack argument to the dword global DAT_01cbea0c and returns with ret 4. |
| `01506d10` | CommandObject_InstallActionMap::ScalarDeletingDtorThunk | likely | Secondary-vtable thunk: subtracts 8 from ECX (sub ecx,8) and jumps to 01506d20. |

**CommandObject_IntroduceCard** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01506390` | CommandObject_IntroduceCard::Dtor | confirmed | Destructor body: stores the CommandObject_IntroduceCard vftables at +0 and +8, then calls base destructor 0142af20. No owned member is released in this body. |
| `01506400` | CommandObject_IntroduceCard::GetTypeName | confirmed | Returns the address of the literal CommandObject_IntroduceCard (mov eax,imm32; ret). |
| `01506440` | CommandObject_IntroduceCard::DoCommand | confirmed | Calls 013508b0 with this+0x30 and the literal CommandObject_IntroduceCard::doCommand, then returns 1. No game state is changed in this body. |
| `01506470` | CommandObject_IntroduceCard::Read | confirmed | Reads the IntroduceCard body: calls slot [4] of this with the stream and a parameter slot (writing its result back), checks the stream with 0142a880 (assert fa… |
| `015064d0` | CommandObject_IntroduceCard::Write | confirmed | Writes the IntroduceCard body: calls slot [2] of this, a no-op trace 013c26e0, the base body 0142acd0, a second no-op trace, then slot [3] of this. |
| `01506530` | CommandObject_IntroduceCard::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls destructor body 01506390, frees the block with _free when bit 0 of the flag argument is set, and returns this. |
| `01506550` | CommandObject_IntroduceCard::Ctor | confirmed | Default constructor: calls base ctor 0142b070, stores CommandObject_IntroduceCard vftables at +0 and +8, assigns the name string CommandObject_IntroduceCard to… |
| `01506410` | CommandObject_IntroduceCard::GetGlobalDword | likely | Returns the dword global DAT_01cbea08 (mov eax,[global]; ret). |
| `01506420` | CommandObject_IntroduceCard::SetGlobalDword | likely | Stores its stack argument to the dword global DAT_01cbea08 and returns with ret 4. |
| `01506520` | CommandObject_IntroduceCard::ScalarDeletingDtorThunk | likely | Secondary-vtable thunk: subtracts 8 from ECX (sub ecx,8) and jumps to 01506530. |

**CommandObject_Kill** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01505df0` | CommandObject_Kill::Dtor | confirmed | Destructor body: stores the CommandObject_Kill vftables at +0 and +8; if the pointer at +0x124 is non-null calls its virtual slot 0 with 1 and clears it; then … |
| `01505e90` | CommandObject_Kill::GetTypeName | confirmed | Returns the address of the literal CommandObject_Kill (mov eax,imm32; ret). |
| `01505fa0` | CommandObject_Kill::Read | confirmed | Reads the Kill body from the stream: calls slot [4] of this with the stream and a local, checks the stream (0142a880), reads one int (013c24d0) and, when non-z… |
| `015060d0` | CommandObject_Kill::Write | confirmed | Writes the Kill body to the stream: calls slot [2] of this, a no-op trace 013c26e0, the base body 0142acd0, the trace text Discard Command, the id from 0134037… |
| `01506180` | CommandObject_Kill::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the destructor body 01505df0, frees the block with _free when bit 0 of the flag argument is set, and returns this. |
| `015061a0` | CommandObject_Kill::Ctor | confirmed | Default constructor: calls base ctor 0142b070, stores CommandObject_Kill vftables at +0 and +8, assigns the name string CommandObject_Kill to +0x48, zeroes +0x… |
| `01506220` | CommandObject_Kill::CtorWithBuffer | confirmed | Constructor overload: calls base ctor 0142b150 with its argument, stores the CommandObject_Kill vftables, assigns the name string, sets +0x68 from 01381460, +0… |
| `01505ea0` | CommandObject_Kill::GetGlobalDword | likely | Returns the dword global DAT_01cbea04 (mov eax,[global]; ret). |
| `01505eb0` | CommandObject_Kill::SetGlobalDword | likely | Stores its stack argument to the dword global DAT_01cbea04 and returns with ret 4. |
| `01506170` | CommandObject_Kill::ScalarDeletingDtorThunk | likely | Secondary-vtable thunk: subtracts 8 from ECX (sub ecx,8) and jumps to 01506180; the thunk is the entry at CommandObject_Kill::vftable +0x8 slot [0]. |

**CommandObject_LoseGame** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `015059b0` | CommandObject_LoseGame::~CommandObject_LoseGame | confirmed | Destructor body (ECX=this): resets vftables 0190df34 at +0 and 0190def8 at +8, then calls base teardown 0142af20 with ECX=this. |
| `01505a20` | CommandObject_LoseGame::GetTypeName | confirmed | Virtual slot 11 of secondary vftable 0190def8: returns the address of the string CommandObject_LoseGame (mov eax, 0190dfa8; ret). |
| `01505a80` | CommandObject_LoseGame::doCommand | confirmed | Virtual slot 17 of primary vftable 0190df34 (thiscall, no stack arguments, returns a dword): calls 01381490 and logs CommandObject_LoseGame::doCommand through … |
| `01505bf0` | CommandObject_LoseGame::DeletingDtor | confirmed | Scalar deleting destructor (ECX=this, one stack byte flag): calls destructor body 015059b0, frees this when bit 0 of the flag is set, returns this. |
| `01505c10` | CommandObject_LoseGame::CommandObject_LoseGame | confirmed | No-argument constructor (ECX=this): calls base init 0142b070, stores vftables 0190df34 at +0 and 0190def8 at +8, zeroes the dword at +0x124, and assigns the 22… |
| `01505c80` | CommandObject_LoseGame::Ctor | confirmed | Constructor: calls base CommandObject ctor 0142b150 with its argument, stores CommandObject_LoseGame::vftable at +0 and +8, assigns the string CommandObject_Lo… |
| `01505af0` | CommandObject_LoseGame::Read | likely | Virtual slot 9 of secondary vftable 0190def8 (thiscall, one argument): calls this->vtbl[+0x10](arg, &local) and asserts via 012f5a60 (file CommandObjectLoseGam… |
| `01505b80` | CommandObject_LoseGame::Write | likely | Virtual slot 10 of secondary vftable 0190def8 (thiscall, one argument): calls this->vtbl[+8](arg), writes the label text Starting CommandObject_LoseGame throug… |

**CommandObject_ModifyProperty** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01504d80` | CommandObject_ModifyProperty::GetTypeName | confirmed | Virtual slot 11 of secondary vftable 0190dc74: returns the address of the string CommandObject_ModifyProperty. |
| `01504f20` | CommandObject_ModifyProperty::undoCommand | confirmed | Virtual slot 19 of primary vftable 0190dcb4 (thiscall, no stack arguments, returns a dword): logs CommandObject_ModifyProperty::undoCommand through 013508b0, l… |
| `01505250` | CommandObject_ModifyProperty::DeletingDtorThunk | confirmed | Thunk in secondary vftable 0190dc74 slot [0]: subtracts 8 from ECX and jumps to 01505260 (the primary scalar deleting destructor). |
| `01505260` | CommandObject_ModifyProperty::DeletingDtor | confirmed | Scalar deleting destructor (ECX=this, one stack byte flag): calls destructor body 01504ce0, frees this when bit 0 of the flag is set, returns this. |
| `01505280` | CommandObject_ModifyProperty::CommandObject_ModifyProperty | confirmed | No-argument constructor (ECX=this): calls base init 0142b070, stores vftables 0190dcb4 at +0 and 0190dc74 at +8, calls 012fa910 twice, assigns the 28-char clas… |
| `01505320` | CommandObject_ModifyProperty::CtorBuildAttributeModifier | confirmed | Constructor overload (ECX=this, stack arguments: a flag byte, value 3, value 4, value 5 and the base argument): calls base init 0142b150 with the last argument… |
| `01504df0` | CommandObject_ModifyProperty::doCommand | likely | Virtual slot 17 of primary vftable 0190dcb4 (thiscall, no stack arguments, returns a dword): gets the card at this+0x68 through 00d8d382 (logs Couldn't get car… |
| `01505060` | CommandObject_ModifyProperty::Read | likely | Virtual slot 9 of secondary vftable 0190dc74 (thiscall, one argument): calls this->vtbl[+0x10](arg, &local) and asserts via 012f5a60 (file CommandObjectModifyP… |
| `01505190` | CommandObject_ModifyProperty::Write | likely | Virtual slot 10 of secondary vftable 0190dc74 (thiscall, one argument): calls this->vtbl[+8](arg), writes label Starting CommandObject_ModifyProperty via 013c2… |
| `01505540` | CommandObject_ModifyProperty::Create | likely | Allocates 0x144 bytes through 00d833a6 (one stack argument), calls the no-argument constructor 01505280 on the block when non-null, and returns block+8 or 0 wh… |

**CommandObject_Move** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01503d90` | CommandObject_Move::~CommandObject_Move | confirmed | Destructor body (ECX=this): resets vftables 0190da3c at +0 and 0190da00 at +8, then calls base teardown 0142af20 with ECX=this. |
| `01503e00` | CommandObject_Move::GetTypeName | confirmed | Virtual slot 11 of secondary vftable 0190da00: returns the address of the string CommandObject_Move (mov eax, 0190dab0; ret). |
| `01504230` | CommandObject_Move::DeletingDtorThunk | confirmed | Thunk in secondary vftable 0190da00 slot [0]: subtracts 8 from ECX and jumps to 01504240 (the primary scalar deleting destructor). |
| `01504240` | CommandObject_Move::DeletingDtor | confirmed | Scalar deleting destructor (ECX=this, one stack byte flag): calls destructor body 01503d90, frees this when bit 0 of the flag is set, returns this. |
| `015043e0` | CommandObject_Move::CommandObject_Move | confirmed | No-argument constructor (ECX=this): calls base init 0142b070, stores vftables 0190da3c and 0190da00, assigns the 18-char class-name string to +0x48, and zeroes… |
| `01504480` | CommandObject_Move::CtorWithActionCode | confirmed | Constructor overload (ECX=this, two stack arguments): calls base init 0142b150 with the first argument, stores vftables 0190da3c and 0190da00 and the class-nam… |
| `01503f30` | CommandObject_Move::Read | likely | Virtual slot 9 of secondary vftable 0190da00 (thiscall, one argument): calls this->vtbl[+0x10](arg, &arg) and asserts via 012f5a60 (file CommandObjectMove.cpp,… |
| `01504120` | CommandObject_Move::Write | likely | Virtual slot 10 of secondary vftable 0190da00 (thiscall, one argument): calls this->vtbl[+8](arg), writes label Starting CommandObject_Move, calls base run 014… |
| `01504770` | CommandObject_Move::Create | likely | Allocates 0x13c bytes through 00d833a6 (one stack argument), calls the no-argument constructor 015043e0 on the block when non-null, and returns block+8 or 0 wh… |
| `01504810` | CommandObject_Move::doCommand | likely | Virtual slot 17 of primary vftable 0190da3c (thiscall, no stack arguments, returns 1): asserts mOriginatingCard (line 0xaf) when null; passes the card's vtable… |

**CommandObject_MultiAction** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `015034f0` | CommandObject_MultiAction::~CommandObject_MultiAction | confirmed | Destructor body (ECX=this): resets vftables 0190d894 at +0 and 0190d858 at +8, then calls base teardown 0142af20 with ECX=this. |
| `01503560` | CommandObject_MultiAction::GetTypeName | confirmed | Virtual slot 11 of secondary vftable 0190d858: returns the address of the string CommandObject_MultiAction (mov eax, 0190d908; ret). |
| `015036e0` | CommandObject_MultiAction::DeletingDtor | confirmed | Scalar deleting destructor (ECX=this, one stack byte flag): calls destructor body 015034f0, frees this when bit 0 of the flag is set, returns this. |
| `01503700` | CommandObject_MultiAction::CommandObject_MultiAction | confirmed | No-argument constructor (ECX=this): calls base init 0142b070, stores vftables 0190d894 at +0 and 0190d858 at +8, assigns the 25-char class-name string to +0x48… |
| `01503790` | CommandObject_MultiAction::CtorWithCustomCommands | confirmed | Constructor overload (ECX=this, eight stack arguments): calls base init 0142b150 with the last argument, stores vftables 0190d894 and 0190d858, assigns the cla… |
| `01503670` | CommandObject_MultiAction::Write | likely | Virtual slot 10 of secondary vftable 0190d858 (thiscall, one argument): calls this->vtbl[+8](arg), writes label Starting CommandObject_MultiAction via 013c26e0… |
| `01503ab0` | CommandObject_MultiAction::Create | likely | Allocates 0x128 bytes through 00d833a6 (one stack argument), calls the no-argument constructor 01503700 on the block when non-null, and returns block+8 or 0 wh… |

**CommandObject_MultiPlayer** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01503110` | CommandObject_MultiPlayer::~CommandObject_MultiPlayer | confirmed | Destructor body (ECX=this): resets vftables 0190d744 at +0 and 0190d704 at +8, then calls base teardown 0142af20 with ECX=this. |
| `01503180` | CommandObject_MultiPlayer::GetTypeName | confirmed | Virtual slot 11 of secondary vftable 0190d704: returns the address of the string CommandObject_MultiPlayer (mov eax, 0190d7b8; ret). |
| `01503290` | CommandObject_MultiPlayer::DeletingDtorThunk | confirmed | Thunk in secondary vftable 0190d704 slot [0]: subtracts 8 from ECX and jumps to 015032a0 (the primary scalar deleting destructor). |
| `015032a0` | CommandObject_MultiPlayer::DeletingDtor | confirmed | Scalar deleting destructor (ECX=this, one stack byte flag): calls destructor body 01503110, frees this when bit 0 of the flag is set, returns this. |
| `015032c0` | CommandObject_MultiPlayer::CommandObject_MultiPlayer | confirmed | No-argument constructor (ECX=this): calls base init 0142b070, stores vftables 0190d744 at +0 and 0190d704 at +8, assigns the 25-char class-name string to +0x48… |
| `01503340` | CommandObject_MultiPlayer::CtorWithThreeArgs | confirmed | Constructor overload (ECX=this, three stack arguments): calls base init 0142b150 with the fourth parameter, stores vftables 0190d744 and 0190d704, assigns the … |
| `015031e0` | CommandObject_MultiPlayer::Read | likely | Virtual slot 9 of secondary vftable 0190d704 (thiscall, one argument): calls this->vtbl[+0x10](arg, &arg) and asserts via 012f5a60 (file CommandObjectMultiPlay… |
| `01503240` | CommandObject_MultiPlayer::Write | likely | Virtual slot 10 of secondary vftable 0190d704 (thiscall, one argument): calls this->vtbl[+8](arg), writes label Starting CommandObject_MultiPlayer via 013c26e0… |
| `01503450` | CommandObject_MultiPlayer::Create | likely | Allocates 0x124 bytes through 00d833a6 (one stack argument), calls the no-argument constructor 015032c0 on the block when non-null, and returns block+8 or 0 wh… |

**CommandObject_Pause** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01502d10` | CommandObject_Pause::~CommandObject_Pause | confirmed | Destructor body (ECX=this): resets vftables 0190d60c at +0 and 0190d5cc at +8, calls 01300cd0 (member teardown), then calls base teardown 0142af20. |
| `01502da0` | CommandObject_Pause::GetTypeName | confirmed | Virtual slot 11 of secondary vftable 0190d5cc: returns the address of the string CommandObject_Pause (mov eax, 0190d680; ret). |
| `01502f30` | CommandObject_Pause::DeletingDtor | confirmed | Scalar deleting destructor (ECX=this, one stack byte flag): calls destructor body 01502d10, frees this when bit 0 of the flag is set, returns this. |
| `01502f50` | CommandObject_Pause::CommandObject_Pause | confirmed | No-argument constructor (ECX=this): calls base init 0142b070, stores vftables 0190d60c at +0 and 0190d5cc at +8, calls 012fa910, and assigns the 19-char class-… |
| `01502fd0` | CommandObject_Pause::CtorWithTwoArgs | confirmed | Constructor overload (ECX=this, two stack arguments): calls base init 0142b150 with the second argument, stores vftables 0190d60c and 0190d5cc, calls 012fa910,… |
| `01502de0` | CommandObject_Pause::Read | likely | Virtual slot 9 of secondary vftable 0190d5cc (thiscall, one argument): calls this->vtbl[+0x10](arg, &arg) and asserts via 012f5a60 (file CommandObjectPause.cpp… |
| `01502e70` | CommandObject_Pause::Write | likely | Virtual slot 10 of secondary vftable 0190d5cc (thiscall, one argument): calls this->vtbl[+8](arg), writes label Starting CommandObject_Pause via 013c26e0, call… |
| `01503070` | CommandObject_Pause::Create | likely | Allocates 0x130 bytes through 00d833a6 (one stack argument), calls the no-argument constructor 01502f50 on the block when non-null, and returns block+8 or 0 wh… |

**CommandObject_PickList** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01502600` | CommandObject_PickList::~CommandObject_PickList | confirmed | Destructor body (ECX=this): resets vftables 0190d4c4 at +0 and 0190d484 at +8, then calls base teardown 0142af20 with ECX=this. |
| `01502670` | CommandObject_PickList::GetTypeName | confirmed | Virtual slot 11 of secondary vftable 0190d484: returns the address of the string CommandObject_PickList (mov eax, 0190d538; ret). |
| `01502790` | CommandObject_PickList::DeletingDtor | confirmed | Scalar deleting destructor (ECX=this, one stack byte flag): calls destructor body 01502600, frees this when bit 0 of the flag is set, returns this. |
| `015027b0` | CommandObject_PickList::CommandObject_PickList | confirmed | No-argument constructor (ECX=this): calls base init 0142b070, stores vftables 0190d4c4 at +0 and 0190d484 at +8, assigns the 22-char class-name string to +0x48… |
| `015028e0` | CommandObject_PickList::CtorWithPickValues | confirmed | Constructor overload (ECX=this, three stack arguments): calls base init 0142b150 with the first argument, stores vftables 0190d4c4 and 0190d484, assigns the cl… |
| `015026d0` | CommandObject_PickList::Read | likely | Virtual slot 9 of secondary vftable 0190d484 (thiscall, one argument): calls this->vtbl[+0x10](arg, &arg) and asserts via 012f5a60 (file CommandObjectPickList.… |
| `01502730` | CommandObject_PickList::Write | likely | Virtual slot 10 of secondary vftable 0190d484 (thiscall, one argument): calls this->vtbl[+8](arg), writes label Starting CommandObject_PickList via 013c26e0, c… |
| `01502840` | CommandObject_PickList::Create | likely | Allocates 0x124 bytes through 00d833a6 (one stack argument), calls the no-argument constructor 015027b0 on the block when non-null, and returns block+8 or 0 wh… |

**CommandObject_PlayCard** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01501d70` | CommandObject_PlayCard::~CommandObject_PlayCard | confirmed | Destructor body (ECX=this): resets vftables 0190d23c at +0 and 0190d200 at +8, then calls base teardown 0142af20 with ECX=this. |
| `01501de0` | CommandObject_PlayCard::GetTypeName | confirmed | Virtual slot 11 of secondary vftable 0190d200: returns the address of the string CommandObject_PlayCard (mov eax, 0190d2b0; ret). |
| `015022f0` | CommandObject_PlayCard::DeletingDtorThunk | confirmed | Thunk in secondary vftable 0190d200 slot [0]: subtracts 8 from ECX and jumps to 01502300 (the primary scalar deleting destructor). |
| `01502300` | CommandObject_PlayCard::DeletingDtor | confirmed | Scalar deleting destructor (ECX=this, one stack byte flag): calls destructor body 01501d70, frees this when bit 0 of the flag is set, returns this. |
| `01502320` | CommandObject_PlayCard::CommandObject_PlayCard | confirmed | No-argument constructor (ECX=this): calls base init 0142b070, stores vftables 0190d23c at +0 and 0190d200 at +8, assigns the 22-char class-name string to +0x48… |
| `015023b0` | CommandObject_PlayCard::CtorWithEnvironment | confirmed | Constructor overload (ECX=this, one stack argument): calls base init 0142b150 with the argument, stores vftables 0190d23c and 0190d200, assigns the class-name … |
| `01502230` | CommandObject_PlayCard::Read | likely | Virtual slot 9 of secondary vftable 0190d200 (thiscall, one argument): calls this->vtbl[+0x10](arg, &arg) and asserts via 012f5a60 (file CommandObjectPlayCard.… |
| `015022a0` | CommandObject_PlayCard::Write | likely | Virtual slot 10 of secondary vftable 0190d200 (thiscall, one argument): calls this->vtbl[+8](arg), writes label Starting CommandObject_PlayCard via 013c26e0, c… |
| `01502560` | CommandObject_PlayCard::Create | likely | Allocates 0x124 bytes through 00d833a6 (one stack argument), calls the no-argument constructor 01502320 on the block when non-null, and returns block+8 or 0 wh… |

**CommandObject_Prevent** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01501920` | CommandObject_Prevent::~CommandObject_Prevent | confirmed | Destructor body (ECX=this): resets vftables 0190d0fc at +0 and 0190d0bc at +8, calls 01300cd0 with ECX=this+0x124, then calls base teardown 0142af20. |
| `015019b0` | CommandObject_Prevent::GetTypeName | confirmed | Virtual slot 11 of secondary vftable 0190d0bc: returns the address of the string CommandObject_Prevent (mov eax, 0190d170; ret). |
| `01501b50` | CommandObject_Prevent::DeletingDtorThunk | confirmed | Thunk in secondary vftable 0190d0bc slot [0]: subtracts 8 from ECX and jumps to 01501b60 (the primary scalar deleting destructor). |
| `01501b60` | CommandObject_Prevent::DeletingDtor | confirmed | Scalar deleting destructor (ECX=this, one stack byte flag): calls destructor body 01501920, frees this when bit 0 of the flag is set, returns this. |
| `01501b80` | CommandObject_Prevent::CommandObject_Prevent | confirmed | No-argument constructor (ECX=this): calls base init 0142b070, stores vftables 0190d0fc at +0 and 0190d0bc at +8, calls 012fa910, assigns the 21-char class-name… |
| `01501c10` | CommandObject_Prevent::CtorWithThreeArgs | confirmed | Constructor overload (ECX=this, three stack arguments): calls base init 0142b150 with the fourth parameter, stores vftables 0190d0fc and 0190d0bc, calls 012fa9… |
| `01501a10` | CommandObject_Prevent::Read | likely | Virtual slot 9 of secondary vftable 0190d0bc (thiscall, one argument): calls this->vtbl[+0x10](arg, &local) and asserts via 012f5a60 (file CommandObjectPrevent… |
| `01501ad0` | CommandObject_Prevent::Write | likely | Virtual slot 10 of secondary vftable 0190d0bc (thiscall, one argument): calls this->vtbl[+8](arg), writes label Starting CommandObject_Prevent via 013c26e0, ca… |
| `01501cd0` | CommandObject_Prevent::Create | likely | Allocates 0x134 bytes through 00d833a6 (one stack argument), calls the no-argument constructor 01501b80 on the block when non-null, and returns block+8 or 0 wh… |

**CommandObject_PushPhase** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `015014d0` | CommandObject_PushPhase::~CommandObject_PushPhase | confirmed | Destructor body (ECX=this): resets vftables 0190cfb4 at +0 and 0190cf74 at +8 (its own class, before the base call), then calls base teardown 0142af20 with ECX… |
| `01501540` | CommandObject_PushPhase::GetTypeName | confirmed | Virtual slot 11 of secondary vftable 0190cf74: returns the address of the string CommandObject_PushPhase (mov eax, 0190d028; ret). |
| `01501650` | CommandObject_PushPhase::DeletingDtorThunk | confirmed | Thunk in secondary vftable 0190cf74 slot [0]: subtracts 8 from ECX and jumps to 01501660 (the primary scalar deleting destructor). |
| `01501660` | CommandObject_PushPhase::DeletingDtor | confirmed | Scalar deleting destructor (ECX=this, one stack byte flag): calls destructor body 015014d0, frees this when bit 0 of the flag is set, returns this. |
| `01501680` | CommandObject_PushPhase::CommandObject_PushPhase | confirmed | No-argument constructor (ECX=this): calls base init 0142b070, stores vftables 0190cfb4 at +0 and 0190cf74 at +8, assigns the 23-char class-name string to +0x48… |
| `015017a0` | CommandObject_PushPhase::CtorWithIntegerValue | confirmed | Constructor overload (ECX=this, three stack arguments): calls base init 0142b150 with the first argument, stores vftables 0190cfb4 and 0190cf74, assigns the cl… |
| `015015a0` | CommandObject_PushPhase::Read | likely | Virtual slot 9 of secondary vftable 0190cf74 (thiscall, one argument): calls this->vtbl[+0x10](arg, &arg) and asserts via 012f5a60 (file CommandObjectPushPhase… |
| `01501600` | CommandObject_PushPhase::Write | likely | Virtual slot 10 of secondary vftable 0190cf74 (thiscall, one argument): calls this->vtbl[+8](arg), writes label Starting CommandObject_PushPhase via 013c26e0, … |
| `01501700` | CommandObject_PushPhase::Create | likely | Allocates 0x124 bytes through 00d833a6 (one stack argument), calls the no-argument constructor 01501680 on the block when non-null, and returns block+8 or 0 wh… |

**CommandObject_ReadyCard** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01501060` | CommandObject_ReadyCard::~CommandObject_ReadyCard | confirmed | Destructor body (ECX=this): resets vftables 0190ce6c at +0 and 0190ce30 at +8, then calls base teardown 014fc250 with ECX=this and returns. |
| `015010d0` | CommandObject_ReadyCard::GetTypeName | confirmed | Virtual slot 11 of secondary vftable 0190ce30: returns the address of the string CommandObject_ReadyCard (mov eax, 0190cee0; ret). |
| `015012c0` | CommandObject_ReadyCard::DeletingDtorThunk | confirmed | Thunk in secondary vftable 0190ce30 slot [0]: subtracts 8 from ECX and jumps to 015012d0 (the primary scalar deleting destructor). |
| `015012d0` | CommandObject_ReadyCard::DeletingDtor | confirmed | Scalar deleting destructor (ECX=this, one stack byte flag): calls destructor body 01501060, frees this when bit 0 of the flag is set, returns this. |
| `015012f0` | CommandObject_ReadyCard::CommandObject_ReadyCard | confirmed | No-argument constructor (ECX=this): calls base init 014fc520 with ECX=this, stores vftables 0190ce6c at +0 and 0190ce30 at +8, assigns the 23-char class-name s… |
| `01501370` | CommandObject_ReadyCard::CtorWithBaseArg | confirmed | Constructor overload (ECX=this, one stack argument): calls 014fc5b0 with 0x309, 0 and the argument before the vftable stores; stores vftables 0190ce6c and 0190… |
| `01501110` | CommandObject_ReadyCard::doCommand | likely | Virtual slot 17 of primary vftable 0190ce6c (thiscall, one stack argument, ret 4): if this+0x68 is nonzero and 014fc780(arg) returns 1, allocates 0x14 bytes, b… |
| `01501210` | CommandObject_ReadyCard::Read | likely | Virtual slot 9 of secondary vftable 0190ce30 (thiscall, one argument): calls this->vtbl[+0x10](arg, &arg) and asserts via 012f5a60 (file CommandObjectReadyCard… |
| `01501270` | CommandObject_ReadyCard::Write | likely | Virtual slot 10 of secondary vftable 0190ce30 (thiscall, one argument): calls this->vtbl[+8](arg), writes label Starting CommandObject_ReadyCard via 013c26e0, … |
| `01501430` | CommandObject_ReadyCard::Create | likely | Allocates 0x138 bytes through 00d833a6 (one stack argument), calls the no-argument constructor 015012f0 on the block when non-null, and returns block+8 or 0 wh… |

**CommandObject_RemoveAttrModFilter** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01500cd0` | CommandObject_RemoveAttrModFilter::~CommandObject_RemoveAtt… | confirmed | Destructor body (ECX=this): resets vftables 0190ccfc at +0 and 0190ccc0 at +8 before calling base teardown 0142af20 with ECX=this. |
| `01500d40` | CommandObject_RemoveAttrModFilter::GetTypeName | confirmed | Virtual slot 11 of secondary vftable 0190ccc0: returns the address of the string CommandObject_RemoveAttrModFilter (mov eax, 0190cd70; ret). |
| `01500e90` | CommandObject_RemoveAttrModFilter::DeletingDtorThunk | confirmed | Thunk in secondary vftable 0190ccc0 slot [0]: subtracts 8 from ECX and jumps to 01500ea0 (the primary scalar deleting destructor). |
| `01500ea0` | CommandObject_RemoveAttrModFilter::DeletingDtor | confirmed | Scalar deleting destructor (ECX=this, one stack byte flag): calls destructor body 01500cd0, frees this when bit 0 of the flag is set, returns this. |
| `01500ec0` | CommandObject_RemoveAttrModFilter::CommandObject_RemoveAttr… | confirmed | No-argument constructor (ECX=this): calls base init 0142b070, stores vftables 0190ccfc at +0 and 0190ccc0 at +8, assigns the 33-char string CommandObject_Remov… |
| `01500f30` | CommandObject_RemoveAttrModFilter::CtorWithBaseArgAndCards | confirmed | Constructor overload (ECX=this, one stack argument, ret 4): calls base init 0142b150 with the argument, stores vftables and the class-name string, then stores … |
| `01500d80` | CommandObject_RemoveAttrModFilter::doCommand | likely | Virtual slot 17 of primary vftable 0190ccfc: dynamic-casts this+0x84 from PlayElement to Card (01381460); passes the result to 00d8d382; returns 0 when that is… |
| `01500de0` | CommandObject_RemoveAttrModFilter::Read | likely | Virtual slot 9 of secondary vftable 0190ccc0 (thiscall, one argument): calls this->vtbl[+0x10](arg, &arg) and asserts via 012f5a60 (file CommandObjectRemoveAtt… |
| `01500e40` | CommandObject_RemoveAttrModFilter::Write | likely | Virtual slot 10 of secondary vftable 0190ccc0 (thiscall, one argument): calls this->vtbl[+8](arg), writes label Starting CommandObject_RemoveAttrModFilter via … |
| `01500fc0` | CommandObject_RemoveAttrModFilter::Create | likely | Allocates 0x124 bytes through 00d833a6 (one stack argument), calls the no-argument constructor 01500ec0 on the block when non-null, and returns block+8 or 0 wh… |

**CommandObject_RemoveInheritedGameText** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01500940` | CommandObject_RemoveInheritedGameText::~CommandObject_Remov… | confirmed | Destructor body: resets the vftables to 0190cb7c at +0 and 0190cb3c at +8 (its own class, before the base call), then calls base teardown 0142af20 with ECX=thi… |
| `015009b0` | CommandObject_RemoveInheritedGameText::GetTypeName | confirmed | Virtual slot 11 of the secondary vftable 0190cb3c: returns the address of the string CommandObject_RemoveInheritedGameText (mov eax, 0190cbf0; ret). |
| `01500b10` | CommandObject_RemoveInheritedGameText::DeletingDtorThunk | confirmed | Thunk in secondary vftable 0190cb3c slot [0]: subtracts 8 from ECX and jumps to 01500b20 (the primary scalar deleting destructor). |
| `01500b20` | CommandObject_RemoveInheritedGameText::DeletingDtor | confirmed | Scalar deleting destructor (ECX=this, one stack byte flag): calls destructor body 01500940, frees this when bit 0 of the flag is set, returns this. |
| `01500b40` | CommandObject_RemoveInheritedGameText::CommandObject_Remove… | confirmed | No-argument constructor (ECX=this): calls base init 0142b070, then stores vftables 0190cb7c at +0 and 0190cb3c at +8, and assigns the 37-char string CommandObj… |
| `01500bb0` | CommandObject_RemoveInheritedGameText::CtorWithBaseArgAndLi… | confirmed | Constructor overload (ECX=this, one stack argument): calls base init 0142b150 with the argument, stores vftables 0190cb7c and 0190cb3c, assigns the class-name … |
| `015009f0` | CommandObject_RemoveInheritedGameText::doCommand | likely | Virtual slot 17 of primary vftable 0190cb7c (thiscall, one stack argument, ret 4): adds 0x84 to ECX, calls 013813e0 on this+0x84, calls the returned object's v… |
| `01500a60` | CommandObject_RemoveInheritedGameText::Read | likely | Virtual slot 9 of secondary vftable 0190cb3c (thiscall, one argument): calls this->vtbl[+0x10](arg, &arg) and asserts via 012f5a60 (file CommandObjectRemoveInh… |
| `01500ac0` | CommandObject_RemoveInheritedGameText::Write | likely | Virtual slot 10 of secondary vftable 0190cb3c (thiscall, one argument): calls this->vtbl[+8](arg), writes label Starting CommandObject_RemoveInheritedGameText … |
| `01500c30` | CommandObject_RemoveInheritedGameText::Create | likely | Allocates 0x124 bytes through 00d833a6 (one stack argument), calls the no-argument constructor 01500b40 on the block when non-null, and returns block+8 or 0 wh… |

**CommandObject_RemoveInstalledAction** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `015002a0` | CommandObject_RemoveInstalledAction::GetTypeName | confirmed | Six bytes (b8 imm32 c3): returns the address of the literal "CommandObject_RemoveInstalledAction" at 0190c950. |
| `015002b0` | CommandObject_RemoveInstalledAction::GetGlobalDword | confirmed | Six bytes (a1 imm32 c3): loads and returns the dword at global 01cbe9cc. |
| `015002c0` | CommandObject_RemoveInstalledAction::SetGlobalDword | confirmed | Twelve bytes: loads the stack argument, stores it to global 01cbe9cc, returns with ret 4. |
| `015002e0` | CommandObject_RemoveInstalledAction::UndoCommand | confirmed | Primary vtable slot [19], returns 1. Traces "CommandObject_RemoveInstalledAction::undoCommand" on this+0x30 through 013508b0 and returns 1. |
| `015003d0` | CommandObject_RemoveInstalledAction::ThunkScalarDeletingDtor | confirmed | Eight-byte thunk at secondary vtable slot [0]: 83 e9 08 subtracts 8 from ECX, then e9 jumps (rel32 +8) to 015003e0, the primary scalar deleting destructor. |
| `015003e0` | CommandObject_RemoveInstalledAction::ScalarDeletingDtor | confirmed | Primary vtable slot [0]. Calls the destructor body 01500230 with ECX=this, calls _free(this) when bit 0 of the flag argument is set, returns this with ret 4. |
| `01500470` | CommandObject_RemoveInstalledAction::CommandObject_RemoveIn… | confirmed | Constructor (ECX=this): calls base init 0142b070 on this, then stores vftables 0190c8dc at +0 and 0190c89c at +8, zeroes the dword at +0x124, and assigns the 3… |
| `015004e0` | CommandObject_RemoveInstalledAction::CtorWithBaseArgAndFlag | confirmed | Second constructor (ECX=this, two stack args, ret 8): calls base init 0142b150 with the first argument, stores the second argument at +0x124, sets the same two… |
| `01500600` | CommandObject_RemoveInstalledAction::doCommand | confirmed | Virtual slot 17 of the primary vftable. Gets the target card (01381490) and origin card (013813e0), logging failures and returning 0 for either missing. Otherw… |
| `01500560` | CommandObject_RemoveInstalledAction::Create | likely | Allocates 0x128 bytes through 00d833a6 (new-style allocator, one stack argument), calls 01500470 on the block when non-null, and returns block+8 (the second ba… |

**CommandObject_RemoveKeyword** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ffe20` | CommandObject_RemoveKeyword::GetTypeName | confirmed | Six bytes (b8 imm32 c3): returns the address of the literal "CommandObject_RemoveKeyword" at 0190c790. |
| `014ffe30` | CommandObject_RemoveKeyword::GetGlobalDword | confirmed | Six bytes (a1 imm32 c3): loads and returns the dword at global 01cbe9c8. |
| `014ffe40` | CommandObject_RemoveKeyword::SetGlobalDword | confirmed | Twelve bytes: loads the stack argument, stores it to global 01cbe9c8, returns with ret 4. |
| `014ffe60` | CommandObject_RemoveKeyword::DoCommand | confirmed | Primary vtable slot [17], returns 1 (0 when the card is null). Casts this+0x68 to Card with 00d8d382; when the cast fails it traces "CARD IS NULL IIN COMMANDOB… |
| `014ffec0` | CommandObject_RemoveKeyword::UndoCommand | confirmed | Primary vtable slot [19], returns 1 (0 when the card is null). Casts this+0x68 to Card with 00d8d382; when the cast fails it traces "Couldn't get card in Comma… |
| `01500010` | CommandObject_RemoveKeyword::ThunkScalarDeletingDtor | confirmed | Eight-byte thunk at secondary vtable slot [0]: 83 e9 08 subtracts 8 from ECX, then e9 jumps (rel32 +8) to 01500020, the primary scalar deleting destructor. |
| `01500020` | CommandObject_RemoveKeyword::ScalarDeletingDtor | confirmed | Primary vtable slot [0]. Calls the destructor body 014ffd90 with ECX=this, calls _free(this) when bit 0 of the flag argument is set, returns this with ret 4. |
| `01500040` | CommandObject_RemoveKeyword::Ctor | confirmed | Default constructor (ECX=this). Stores the primary vtable 0190c71c at +0 and the secondary 0190c6dc at +8, calls 012fa910, assigns the 27-character string "Com… |
| `015000c0` | CommandObject_RemoveKeyword::CtorWithArgs | confirmed | Constructor with three stack arguments (ret 0xc): the first (byte) selects the card lookup, the second goes to 013010e0 and the third goes to 0142b150 and 0130… |

**CommandObject_RemoveProperty** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ff650` | CommandObject_RemoveProperty::GetTypeName | confirmed | Six bytes (b8 imm32 c3): returns the address of the literal "CommandObject_RemoveProperty" at 0190c5e8. |
| `014ff660` | CommandObject_RemoveProperty::GetGlobalDword | confirmed | Six bytes (a1 imm32 c3): loads and returns the dword at global 01cbe9c4. |
| `014ff670` | CommandObject_RemoveProperty::SetGlobalDword | confirmed | Twelve bytes: loads the stack argument, stores it to global 01cbe9c4, returns with ret 4. |
| `014ff8b0` | CommandObject_RemoveProperty::ThunkScalarDeletingDtor | confirmed | Eight-byte thunk at secondary vtable slot [0]: 83 e9 08 subtracts 8 from ECX, then e9 jumps (rel32 +8) to 014ff8c0, the primary scalar deleting destructor. |
| `014ff8c0` | CommandObject_RemoveProperty::ScalarDeletingDtor | confirmed | Primary vtable slot [0]. Calls the destructor body 014ff5b0 with ECX=this, calls _free(this) when bit 0 of the flag argument is set, returns this with ret 4. |
| `014ff8e0` | CommandObject_RemoveProperty::DoCommand | confirmed | Primary vtable slot [17], returns 1 (0 when the card is null). Casts this+0x68 to Card with 00d8d382; a null card traces "CARD IS NULL IIN COMMANDOBJECT_REMOVE… |
| `014ffb60` | CommandObject_RemoveProperty::Ctor | confirmed | Default constructor (ECX=this). Stores the primary vtable 0190c574 at +0 and the secondary 0190c538 at +8, calls 012fa910 twice (the two inline members at +0x1… |
| `014ffc00` | CommandObject_RemoveProperty::CtorWithArgs | confirmed | Constructor with three stack arguments (ret 0xc): the first (byte) selects the card lookup, the second is the property id or flag tested against zero, the thir… |

**CommandObject_RemoveSuppressEffect** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ff1a0` | CommandObject_RemoveSuppressEffect::GetTypeName | confirmed | Six bytes (b8 imm32 c3): returns the address of the literal "CommandObject_RemoveSuppressEffect" at 0190c448. |
| `014ff1b0` | CommandObject_RemoveSuppressEffect::GetGlobalDword | confirmed | Six bytes (a1 imm32 c3): loads and returns the dword at global 01cbe9c0. |
| `014ff1c0` | CommandObject_RemoveSuppressEffect::SetGlobalDword | confirmed | Twelve bytes: loads the stack argument, stores it to global 01cbe9c0, returns with ret 4. |
| `014ff1f0` | CommandObject_RemoveSuppressEffect::DoCommand | confirmed | Primary vtable slot [17], returns 1. Asserts mTarget (this+0x68 non-null, CommandObjectRemoveSuppressEffect.cpp line 0x2c) and calls 01342250 with this+0x124 a… |
| `014ff230` | CommandObject_RemoveSuppressEffect::UndoCommand | confirmed | Primary vtable slot [19], returns 1. Calls 01383720; when it returns zero it traces "Will Expire set to false on environment." on this+0x84 and returns. Otherw… |
| `014ff390` | CommandObject_RemoveSuppressEffect::ThunkScalarDeletingDtor | confirmed | Eight-byte thunk at secondary vtable slot [0]: 83 e9 08 subtracts 8 from ECX, then e9 jumps (rel32 +8) to 014ff3a0, the primary scalar deleting destructor. |
| `014ff3a0` | CommandObject_RemoveSuppressEffect::ScalarDeletingDtor | confirmed | Primary vtable slot [0]. Calls the destructor body 014ff110 with ECX=this, calls _free(this) when bit 0 of the flag argument is set, returns this with ret 4. |
| `014ff3c0` | CommandObject_RemoveSuppressEffect::Ctor | confirmed | Default constructor (ECX=this). Stores the primary vtable 0190c3d4 at +0 and the secondary 0190c394 at +8, calls 012fa910, assigns the 34-character string "Com… |
| `014ff440` | CommandObject_RemoveSuppressEffect::CtorWithArgs | confirmed | Constructor with three stack arguments (ret 0xc): the first (byte) selects the card lookup, the second goes to 013010e0 and the third goes to 0142b150 and 0130… |

**CommandObject_RemoveToken** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014fe8c0` | CommandObject_RemoveToken::GetTypeName | confirmed | Six bytes (b8 imm32 c3): returns the address of the literal "CommandObject_RemoveToken" at 0190c254. |
| `014fe8d0` | CommandObject_RemoveToken::GetGlobalDword | confirmed | Six bytes (a1 imm32 c3): loads and returns the dword at global 01cbe9bc. |
| `014fe8e0` | CommandObject_RemoveToken::SetGlobalDword | confirmed | Twelve bytes: loads the stack argument, stores it to global 01cbe9bc, returns with ret 4. |
| `014fe940` | CommandObject_RemoveToken::DoCommand | confirmed | Primary vtable slot [17], returns a byte. If this+0x68 (card) is null it asserts mTarget (CommandObjectRemoveToken.cpp line 0x5f) and continues. Casts this+0x6… |
| `014fed80` | CommandObject_RemoveToken::ThunkScalarDeletingDtor | confirmed | Eight-byte thunk at secondary vtable slot [0]: 83 e9 08 subtracts 8 from ECX, then e9 jumps (rel32 +8) to 014fed90, the primary scalar deleting destructor. |
| `014fed90` | CommandObject_RemoveToken::ScalarDeletingDtor | confirmed | Primary vtable slot [0]. Calls the destructor body 014fe830 with ECX=this, calls _free(this) when bit 0 of the flag argument is set, returns this with ret 4. |
| `014fedb0` | CommandObject_RemoveToken::Ctor | confirmed | Default constructor (ECX=this). Stores the primary vtable 0190c1dc at +0 and the secondary 0190c19c at +8, calls 012fa910, assigns the 25-character string "Com… |
| `014fee50` | CommandObject_RemoveToken::CtorWithArgs | confirmed | Constructor with four stack arguments (ret 0x10): the first (byte) selects the card lookup, the second is stored at +0x130, the third goes to 013010e0 and the … |
| `014fef60` | CommandObject_RemoveToken::CtorWithArgsNoToken | confirmed | Constructor with four stack arguments (ret 0x10), the same shape as CommandObject_RemoveToken::CtorWithArgs except that the second argument is stored at +0x138… |
| `014fe900` | CommandObject_RemoveToken::ReturnDwordAt7c | likely | Four bytes (8b 41 7c c3): returns the dword at this+0x7c. |
| `014fe910` | CommandObject_RemoveToken::ReturnDwordAt74 | likely | Four bytes: returns the dword at this+0x74. |

**CommandObject_Reshuffle** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014fe4e0` | CommandObject_Reshuffle::GetTypeName | confirmed | Six bytes (b8 imm32 c3): returns the address of the literal "CommandObject_Reshuffle" at 0190c108. |
| `014fe4f0` | CommandObject_Reshuffle::GetGlobalDword | confirmed | Six bytes (a1 imm32 c3): loads and returns the dword at global 01cbe9b8. |
| `014fe500` | CommandObject_Reshuffle::SetGlobalDword | confirmed | Twelve bytes: loads the stack argument, stores it to global 01cbe9b8, returns with ret 4. |
| `014fe5f0` | CommandObject_Reshuffle::ThunkScalarDeletingDtor | confirmed | Eight-byte thunk at secondary vtable slot [0]: 83 e9 08 subtracts 8 from ECX, then e9 jumps (rel32 +8) to 014fe600, the primary scalar deleting destructor. |
| `014fe600` | CommandObject_Reshuffle::ScalarDeletingDtor | confirmed | Primary vtable slot [0]. Calls the destructor body 014fe470 with ECX=this, calls _free(this) when bit 0 of the flag argument is set, returns this with ret 4. |
| `014fe620` | CommandObject_Reshuffle::Ctor | confirmed | Default constructor (ECX=this). Stores the primary vtable 0190c094 at +0 and the secondary 0190c058 at +8, assigns the 23-character string "CommandObject_Reshu… |
| `014fe6b0` | CommandObject_Reshuffle::CtorWithArgs | confirmed | Constructor with one stack argument (ret 4). Calls 0142b150 with the argument, stores both vtables, assigns the class-name string at +0x48, sets the byte at +0… |

**CommandObject_Reveal** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014fdbe0` | CommandObject_Reveal::GetTypeName | confirmed | Six bytes (b8 imm32 c3): returns the address of the literal "CommandObject_Reveal" at 0190bfc8. |
| `014fdbf0` | CommandObject_Reveal::GetGlobalDword | confirmed | Six bytes (a1 imm32 c3): loads and returns the dword at global 01cbe9b4. |
| `014fdc00` | CommandObject_Reveal::SetGlobalDword | confirmed | Twelve bytes: loads the stack argument, stores it to global 01cbe9b4, returns with ret 4. |
| `014fdd00` | CommandObject_Reveal::ThunkScalarDeletingDtor | confirmed | Eight-byte thunk at secondary vtable slot [0]: 83 e9 08 subtracts 8 from ECX, then e9 jumps (rel32 +8) to 014fdd10, the primary scalar deleting destructor. |
| `014fdd10` | CommandObject_Reveal::ScalarDeletingDtor | confirmed | Primary vtable slot [0]. Calls the destructor body 014fdb70 with ECX=this, calls _free(this) when bit 0 of the flag argument is set, returns this with ret 4. |
| `014fdd30` | CommandObject_Reveal::Ctor | confirmed | Default constructor (ECX=this). Stores the primary vtable 0190bf54 at +0 and the secondary 0190bf18 at +8, assigns the 20-character string "CommandObject_Revea… |
| `014fddc0` | CommandObject_Reveal::CtorWithArgs | confirmed | Constructor with four stack arguments (ret 0x10). Calls 0142b150 with the last argument, stores both vtables, assigns the class-name string at +0x48, sets the … |
| `014fdf80` | CommandObject_Reveal::CtorFromValues | likely | Constructor with five stack arguments. Calls 0142b150 with the last argument, stores both vtables, assigns the class-name string at +0x48, sets the byte at +0x… |

**CommandObject_SendCardIntroductions** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014fd830` | CommandObject_SendCardIntroductions::GetTypeName | confirmed | Six bytes (b8 imm32 c3): returns the address of the literal "CommandObject_SendCardIntroductions" at 0190be50. |
| `014fd840` | CommandObject_SendCardIntroductions::GetGlobalDword | confirmed | Six bytes (a1 imm32 c3): loads and returns the dword at global 01cbe9b0. |
| `014fd850` | CommandObject_SendCardIntroductions::SetGlobalDword | confirmed | Twelve bytes: loads the stack argument, stores it to global 01cbe9b0, returns with ret 4. |
| `014fd940` | CommandObject_SendCardIntroductions::ThunkScalarDeletingDtor | confirmed | Eight-byte thunk at secondary vtable slot [0]: 83 e9 08 subtracts 8 from ECX, then e9 jumps (rel32 +8) to 014fd950, the primary scalar deleting destructor. |
| `014fd950` | CommandObject_SendCardIntroductions::ScalarDeletingDtor | confirmed | Primary vtable slot [0]. Calls the destructor body 014fd7c0 with ECX=this, calls _free(this) when bit 0 of the flag argument is set, returns this with ret 4. |
| `014fd970` | CommandObject_SendCardIntroductions::Ctor | confirmed | Default constructor (ECX=this). Stores the primary vtable 0190bddc at +0 and the secondary 0190bd9c at +8, assigns the 35-character string "CommandObject_SendC… |
| `014fd9e0` | CommandObject_SendCardIntroductions::CtorWithArgs | confirmed | Constructor with two stack arguments (ret 8). Calls 0142b150 with the second argument, stores both vtables, assigns the class-name string at +0x48, sets the by… |

**CommandObject_SetPhaseDialog** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014fd070` | CommandObject_SetPhaseDialog::GetTypeName | confirmed | Six bytes (b8 imm32 c3): returns the address of the literal "CommandObject_SetPhaseDialog" at 0190bce0. |
| `014fd080` | CommandObject_SetPhaseDialog::GetGlobalDword | confirmed | Six bytes (a1 imm32 c3): loads and returns the dword at global 01cbe9ac. |
| `014fd090` | CommandObject_SetPhaseDialog::SetGlobalDword | confirmed | Twelve bytes: loads the stack argument, stores it to global 01cbe9ac, returns with ret 4. |
| `014fd1c0` | CommandObject_SetPhaseDialog::ThunkScalarDeletingDtor | confirmed | Eight-byte thunk at secondary vtable slot [0]: 83 e9 08 subtracts 8 from ECX, then e9 jumps (rel32 +8) to 014fd1d0, the primary scalar deleting destructor. |
| `014fd1d0` | CommandObject_SetPhaseDialog::ScalarDeletingDtor | confirmed | Primary vtable slot [0]. Calls the destructor body 014fcfe0 with ECX=this, calls _free(this) when bit 0 of the flag argument is set, returns this with ret 4. |
| `014fd1f0` | CommandObject_SetPhaseDialog::Ctor | confirmed | Default constructor (ECX=this). Stores the primary vtable 0190bc6c at +0 and the secondary 0190bc30 at +8, calls 012fa910, assigns the 28-character string "Com… |
| `014fd270` | CommandObject_SetPhaseDialog::CtorWithArgs | confirmed | Constructor with two stack arguments (ret 8). Calls 0142b150 with the second argument, stores both vtables, calls 012fa910 on the inline member at object+0x124… |
| `014fd3b0` | CommandObject_SetPhaseDialog::DoCommand | confirmed | Primary vtable slot [17], returns void. Acts only when the integer at this+0x128 equals 14: reads the message value at this+0x12c and looks up entries 0, 1 and… |

**CommandObject_SetProperty** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014fc2e0` | CommandObject_SetProperty::GetTypeName | confirmed | Six bytes (b8 imm32 c3): returns the address of the literal "CommandObject_SetProperty" at 0190ba10. |
| `014fc2f0` | CommandObject_SetProperty::GetGlobalDword | confirmed | Six bytes (a1 imm32 c3): loads and returns the dword at global 01cbe9a8. |
| `014fc300` | CommandObject_SetProperty::SetGlobalDword | confirmed | Twelve bytes: loads the stack argument, stores it to global 01cbe9a8, returns with ret 4. |
| `014fc4f0` | CommandObject_SetProperty::ThunkScalarDeletingDtor | confirmed | Eight-byte thunk at secondary vtable slot [0]: 83 e9 08 subtracts 8 from ECX, then e9 jumps (rel32 +8) to 014fc500, the primary scalar deleting destructor. |
| `014fc500` | CommandObject_SetProperty::ScalarDeletingDtor | confirmed | Primary vtable slot [0]. Calls the destructor body 014fc250 with ECX=this, calls _free(this) when bit 0 of the flag argument is set, returns this with ret 4. |
| `014fc520` | CommandObject_SetProperty::Ctor | confirmed | Default constructor (ECX=this). Stores the primary vtable 0190b99c at +0 and the secondary 0190b960 at +8, calls 012fa910, assigns the 25-character string "Com… |
| `014fcc10` | CommandObject_SetProperty::UndoCommand | confirmed | Primary vtable slot [19]; the same slot is also used by CommandObject_UnreadyCard (0190b1f4) and CommandObject_ReadyCard (0190ce6c). Casts this+0x68 to Card (0… |

**CommandObject_Shuffle** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014fbce0` | CommandObject_Shuffle::Dtor | confirmed | Destructor body called by the scalar deleting destructor 014fbfc0. Stores the primary vtable 0190b834 at +0 and the secondary 0190b7f8 at +8, then runs the EH … |
| `014fbd50` | CommandObject_Shuffle::GetTypeName | confirmed | Six bytes (b8 imm32 c3): returns the address of the literal "CommandObject_Shuffle" at 0190b8a8. |
| `014fbd60` | CommandObject_Shuffle::GetGlobalDword | confirmed | Six bytes (a1 imm32 c3): loads and returns the dword at global 01cbe9a4. |
| `014fbd70` | CommandObject_Shuffle::SetGlobalDword | confirmed | Twelve bytes: loads the stack argument, stores it to global 01cbe9a4, returns with ret 4. |
| `014fbd90` | CommandObject_Shuffle::DoCommand | confirmed | Primary vtable slot [17]. Asserts object+0x128 (mPlayer) is non-null (CommandObjectShuffle.cpp line 0x2e). If object+0x124 is not 6, calls the vtable +0x48 slo… |
| `014fbfb0` | CommandObject_Shuffle::ThunkScalarDeletingDtor | confirmed | Eight-byte thunk at secondary vtable slot [0]: 83 e9 08 subtracts 8 from ECX, then e9 jumps (rel32 +8) to 014fbfc0, the primary scalar deleting destructor. |
| `014fbfc0` | CommandObject_Shuffle::ScalarDeletingDtor | confirmed | Primary vtable slot [0]. Calls the destructor body 014fbce0 with ECX=this, calls _free(this) when bit 0 of the flag argument is set, returns this with ret 4. |
| `014fbfe0` | CommandObject_Shuffle::Ctor | confirmed | Default constructor (ECX=this). Stores the primary vtable 0190b834 at +0 and the secondary 0190b7f8 at +8, assigns the 21-character string "CommandObject_Shuff… |
| `014fc060` | CommandObject_Shuffle::CtorWithArgs | confirmed | Constructor with two stack arguments (ret 8). Calls 0142b150 with the second argument, stores both vtables, assigns the class-name string at +0x48, stores the … |

**CommandObject_SuppressGameText** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014fb8f0` | CommandObject_SuppressGameText::Dtor | confirmed | Destructor body called by the scalar deleting destructor 014fbad0. Stores the primary vtable 0190b6a4 at +0 and the secondary 0190b668 at +8, then runs the EH … |
| `014fb960` | CommandObject_SuppressGameText::GetTypeName | confirmed | Six bytes (b8 imm32 c3): returns the address of the literal "CommandObject_SuppressGameText" at 0190b718. |
| `014fb970` | CommandObject_SuppressGameText::GetGlobalDword | confirmed | Six bytes (a1 imm32 c3): loads and returns the dword at global 01cbe9a0. |
| `014fb980` | CommandObject_SuppressGameText::SetGlobalDword | confirmed | Twelve bytes: loads the stack argument, stores it to global 01cbe9a0, returns with ret 4. |
| `014fb9a0` | CommandObject_SuppressGameText::DoCommand | confirmed | Primary vtable slot [17]. Traces "CommandObject_SuppressGameText::doCommand" on this+0x30, casts this+0x68 from PlayElement to Card with 00d8d382, passes the r… |
| `014fb9e0` | CommandObject_SuppressGameText::UndoCommand | confirmed | Primary vtable slot [19]. Casts this+0x68 from PlayElement to Card with 00d8d382, passes the result to 013568c0 and returns 1. |
| `014fbac0` | CommandObject_SuppressGameText::ThunkScalarDeletingDtor | confirmed | Eight-byte thunk at secondary vtable slot [0]: 83 e9 08 subtracts 8 from ECX, then e9 jumps (rel32 +8) to 014fbad0, the primary scalar deleting destructor. |
| `014fbad0` | CommandObject_SuppressGameText::ScalarDeletingDtor | confirmed | Primary vtable slot [0]. Calls the destructor body 014fb8f0 with ECX=this, calls _free(this) when bit 0 of the flag argument is set, returns this with ret 4. |
| `014fbaf0` | CommandObject_SuppressGameText::Ctor | confirmed | Default constructor (ECX=this). Stores the primary vtable 0190b6a4 at +0 and the secondary 0190b668 at +8, assigns the 30-character string "CommandObject_Suppr… |
| `014fbb70` | CommandObject_SuppressGameText::CtorWithArgs | confirmed | Constructor with one stack argument (ret 4). Calls 0142b150 with the argument, stores both vtables, assigns the class-name string at +0x48, then with ECX = the… |

**CommandObject_UIEffect** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014fb450` | CommandObject_UIEffect::Dtor | confirmed | Destructor body called by the scalar deleting destructor 014fb740. Stores the primary vtable 0190b50c at +0 and the secondary vtable 0190b4cc at +8, then runs … |
| `014fb4c0` | CommandObject_UIEffect::GetTypeName | confirmed | Six bytes (b8 imm32 c3): returns the address of the literal "CommandObject_UIEffect" at 0190b580. |
| `014fb4d0` | CommandObject_UIEffect::GetGlobalDword | confirmed | Six bytes (a1 imm32 c3): loads and returns the dword at global 01cbe99c. |
| `014fb4e0` | CommandObject_UIEffect::SetGlobalDword | confirmed | Twelve bytes: loads the stack argument, stores it to global 01cbe99c, returns with ret 4. |
| `014fb500` | CommandObject_UIEffect::DoCommand | confirmed | Primary vtable slot [17]. Traces "CommandObject_UIEffect::doCommand %d $d" on this+0x30 with this+0x124, allocates 0x14 bytes with 00d833a6 and, when non-null,… |
| `014fb610` | CommandObject_UIEffect::UndoCommand | confirmed | Primary vtable slot [19]. Traces "CommandObject_UIEffect::undoCommand %d $d" on this+0x30 with this+0x124, calls 01381460 and returns whether it is non-zero (b… |
| `014fb730` | CommandObject_UIEffect::AdjustorThunk | confirmed | Eight-byte thunk at secondary vtable slot [0]: subtracts 8 from ECX and jumps to 014fb740 (the primary scalar deleting destructor). |
| `014fb740` | CommandObject_UIEffect::ScalarDeletingDtor | confirmed | Primary vtable slot [0]. Calls the destructor body 014fb450 with ECX=this, calls _free(this) when bit 0 of the flag argument is set, returns this with ret 4. |
| `014fb760` | CommandObject_UIEffect::Ctor | confirmed | Default constructor (ECX=this). Stores the primary vtable 0190b50c at +0 and the secondary 0190b4cc at +8, zeroes the dword at +0x124, assigns the 22-character… |
| `014fb7d0` | CommandObject_UIEffect::CtorWithArgs | confirmed | Constructor with two stack arguments. Calls 0142b150 with the second argument, stores the first argument at +0x124, stores both vtables, assigns the 22-charact… |

**CommandObject_UnInstallActionMap** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014faf20` | CommandObject_UnInstallActionMap::TypeName | confirmed | Returns the C string 'CommandObject_UnInstallActionMap' (no arguments, no side effects). |
| `014faf70` | CommandObject_UnInstallActionMap::DoCommand | confirmed | Primary vtable slot [17]. If this+0x124 is 0, traces "Couldn't get action!" on this+0x30 and returns with the low byte cleared (false). Otherwise traces "Remov… |
| `014fb260` | CommandObject_UnInstallActionMap::ThunkScalarDeletingDtor | confirmed | Eight-byte thunk at secondary vtable slot [0]: 83 e9 08 subtracts 8 from ECX, then e9 jumps (rel32 +8) to 014fb270, the primary scalar deleting destructor. |
| `014fb270` | CommandObject_UnInstallActionMap::ScalarDeletingDtor | confirmed | Primary vtable slot [0]. Calls the destructor body 014faeb0 with ECX=this, calls _free(this) when bit 0 of the flag argument is set, returns this with ret 4. |
| `014fb290` | CommandObject_UnInstallActionMap::Ctor | confirmed | Default constructor (ECX=this). Stores the primary vtable 0190b364 at +0 and the secondary vtable 0190b324 at +8, assigns the 32-character string "CommandObjec… |
| `014fb310` | CommandObject_UnInstallActionMap::CtorWithArgs | confirmed | Constructor with two stack arguments (ret 8). Calls 0142b150 with the second argument, stores both vtables, assigns the class-name string at +0x48, then with E… |
| `014faeb0` | CommandObject_UnInstallActionMap::DtorBody | likely | Fastcall. Stores CommandObject_UnInstallActionMap::vftable (0190b364) at +0 and the secondary (0190b324) at +8, then calls 0142af20 (base destructor body, whos… |
| `014faf30` | CommandObject_UnInstallActionMap::GetClassStatic | likely | Returns the dword at DAT_01cbe998 (no arguments). |
| `014faf40` | CommandObject_UnInstallActionMap::SetClassStatic | likely | Stores its argument into DAT_01cbe998 (one stack argument, void). |

**CommandObject_UnreadyCard** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014faa50` | CommandObject_UnreadyCard::TypeName | confirmed | Returns the C string 'CommandObject_UnreadyCard' (no arguments, no side effects). |
| `014facb0` | CommandObject_UnreadyCard::~CommandObject_UnreadyCard | confirmed | Scalar deleting destructor: calls the CommandObject_UnreadyCard dtor body 014fa9e0, then frees this when bit 0 of the flag argument is set. Returns this. |
| `014facd0` | CommandObject_UnreadyCard::CommandObject_UnreadyCard | confirmed | Fastcall with the new object in ECX. Calls 014fc520 (base constructor) first, stores both CommandObject_UnreadyCard vtables (+0 and +8), assigns the name 'Comm… |
| `014fa9e0` | CommandObject_UnreadyCard::DtorBody | likely | Fastcall. Stores CommandObject_UnreadyCard::vftable (0190b1f4) at +0 and the secondary (0190b1b8) at +8, then calls 014fc250 with the object (base destructor b… |
| `014faa60` | CommandObject_UnreadyCard::GetClassStatic | likely | Returns the dword at DAT_01cbe994 (no arguments). |
| `014faa70` | CommandObject_UnreadyCard::SetClassStatic | likely | Stores its argument into DAT_01cbe994 (one stack argument, void). |
| `014faca0` | CommandObject_UnreadyCardSecondaryDtorThunk | likely | Thunk on the secondary CommandObject_UnreadyCard base: subtracts 8 from ECX and jumps to the scalar deleting destructor 014facb0. Nothing else. |
| `014fad50` | CommandObject_UnreadyCard::ConstructWithCode | likely | Thiscall with one stack argument (base setup). Calls 014fc5b0 with (0x309, 0, argument) first, stores both vtables, assigns the name string, default-constructs… |
| `014fae10` | CommandObject_UnreadyCard::Create | likely | No arguments. Allocates 0x138 bytes with operator new (00d833a6); when the allocation succeeds it calls the default constructor 014facd0 and returns the object… |
| `014fae90` | CommandObject_UnreadyCard::RegisterFactory | likely | Pushes the factory 014fae10 and the constant 0x178, calls 013ce130, moves its EAX into ECX and calls 013ce230, which stores the factory under key 0x178 (ret 8)… |

**CommandObject_WaitFor** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014fa5a0` | CommandObject_WaitFor::TypeName | confirmed | Returns the C string 'CommandObject_WaitFor' (no arguments, no side effects). |
| `014fa6e0` | CommandObject_WaitFor::~CommandObject_WaitFor | confirmed | Scalar deleting destructor: calls the CommandObject_WaitFor dtor body 014fa510, then frees this when bit 0 of the flag argument is set. Returns this. |
| `014fa810` | CommandObject_WaitFor::CommandObject_WaitFor | confirmed | Fastcall with the new object in ECX. Calls 0142b070 (base constructor) first, stores CommandObject_WaitFor::vftable at +0 and +8, default-constructs the ValueD… |
| `014fa510` | CommandObject_WaitFor::DtorBody | likely | Fastcall. Stores CommandObject_WaitFor::vftable (0190b06c) at +0 and the secondary (0190b02c) at +8, calls 01300cd0 on the member at this+0x124 (a ValueData-ty… |
| `014fa5b0` | CommandObject_WaitFor::GetClassStatic | likely | Returns the dword at DAT_01cbe990 (no arguments). |
| `014fa5c0` | CommandObject_WaitFor::SetClassStatic | likely | Stores its argument into DAT_01cbe990 (one stack argument, void). |
| `014fa6d0` | CommandObject_WaitForSecondaryDtorThunk | likely | Thunk on the secondary CommandObject_WaitFor base: adjusts ECX to the primary object and jumps to the scalar deleting destructor 014fa6e0. Nothing else. |
| `014fa700` | CommandObject_WaitFor::DoCommand | likely | Fastcall, receiver = primary object. Traces 'CommandObject_WaitFor::doCommand ' (013846b0). Asserts (012f5a60) that the member mValue at this+0x124 is an int v… |
| `014fa890` | CommandObject_WaitFor::ConstructWithValue | likely | Thiscall with two stack arguments (value param_2 and base setup param_3). Calls 0142b150 with param_3, stores both vtables, default-constructs the ValueData me… |

**CommandObject_While** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014f9c60` | CommandObject_While::DtorBody | confirmed | Fastcall with the object in ECX. Stores CommandObject_While::vftable at +0 and at +8 (the secondary table), then calls 0142af20 (base destructor body; its vtab… |
| `014f9cd0` | CommandObject_While::TypeName | confirmed | Returns the C string 'CommandObject_While' (no arguments, no side effects). |
| `014f9e70` | CommandObject_While::~CommandObject_While | confirmed | Scalar deleting destructor: calls the CommandObject_While dtor body 014f9c60, then frees this when bit 0 of the flag argument is set. Returns this. |
| `014f9e90` | CommandObject_While::CommandObject_While | confirmed | Fastcall with the new object in ECX. Calls 0142b070 (base constructor) first, then stores CommandObject_While::vftable at +0 and +8, assigns the literal 'Comma… |
| `014f9ce0` | CommandObject_While::GetClassStatic | likely | Returns the dword at DAT_01cbe988 (no arguments). |
| `014f9cf0` | CommandObject_While::SetClassStatic | likely | Stores its argument into DAT_01cbe988 (one stack argument, void). |
| `014f9e60` | CommandObject_WhileSecondaryDtorThunk | likely | Thunk on the secondary CommandObject_While base: adjusts ECX to the primary object and jumps to the scalar deleting destructor 014f9e70. Nothing else. |
| `014f9f10` | CommandObject_While::ConstructWithArgs | likely | Thiscall with three arguments after the receiver. Calls 0142b150 with the fourth argument as base setup, stores both CommandObject_While vtables, clears the fl… |

**CommandObject_WinGame** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014fa170` | CommandObject_WinGame::TypeName | confirmed | Returns the C string 'CommandObject_WinGame' (no arguments, no side effects). |
| `014fa330` | CommandObject_WinGame::~CommandObject_WinGame | confirmed | Scalar deleting destructor: calls the CommandObject_WinGame dtor body 014fa100, then frees this when bit 0 of the flag argument is set. Returns this. |
| `014fa350` | CommandObject_WinGame::CommandObject_WinGame | confirmed | Fastcall with the new object in ECX. Calls 0142b070 (base constructor) first, then stores both CommandObject_WinGame vtables (+0 and +8), zeroes the dword at +… |
| `014fa100` | CommandObject_WinGame::DtorBody | likely | Fastcall. Stores CommandObject_WinGame::vftable at +0 and at +8 (secondary), then calls 0142af20 (base destructor body, whose vtables are CommandObject::vftabl… |
| `014fa180` | CommandObject_WinGame::GetClassStatic | likely | Returns the dword at DAT_01cbe98c (no arguments). |
| `014fa190` | CommandObject_WinGame::SetClassStatic | likely | Stores its argument into DAT_01cbe98c (one stack argument, void). |
| `014fa1c0` | CommandObject_WinGame::DoCommand | likely | Fastcall with the receiver in ECX (ret 0 or 1). Calls 01381490 and a trace 01384af0 with the text 'CommandObject_WinGame::doCommand'; returns 0 when 01381490 r… |
| `014fa320` | CommandObject_WinGameSecondaryDtorThunk | likely | Thunk on the secondary CommandObject_WinGame base: adjusts ECX to the primary object and jumps to the scalar deleting destructor 014fa330. Nothing else. |

**ConcatExpression** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0149ced0` | ConcatExpression::ConcatExpression | confirmed | Constructor body: runs the Expression base constructor 0149adc0 and stores the ConcatExpression vtable (018f9e14). |
| `0149cf50` | ConcatExpression::GetTypeName | confirmed | Virtual slot [11] of ConcatExpression. Returns the literal 'ConcatExpression'. |
| `0149cfe0` | ConcatExpression::AssertFalseStub | confirmed | Virtual slot [15] of ConcatExpression. Calls the assert helper 012f5a60 with 'false', source ConcatExpression.cpp and line 0x12, then returns 0. |
| `0149d080` | ConcatExpression::ScalarDeletingDtor | confirmed | Virtual slot [0]: stores the ConcatExpression vtable, calls the base destructor 0149ae80, frees the object with _free when bit 0 of the argument is set. Return… |
| `0149d0e0` | ConcatExpression::Evaluate | confirmed | Virtual slot [14] of ConcatExpression. Traces '(Concat'. Evaluates lhs (+0x1c) and rhs (+0x20) through slot [14]; both must be strings (type 3). Takes the two … |
| `0149cf90` | ConcatExpression::~ConcatExpression | likely | Destructor body without free: stores the ConcatExpression vtable, then calls the base destructor 0149ae80. |
| `0149d000` | ConcatExpression::Create | likely | Factory: operator new 00d833a6 with size 0x24; on null it returns 0, otherwise it constructs with 0149ced0 and returns the object. |
| `0149d060` | ConcatExpression::RegisterFactory | likely | Registers factory 0149d000 under classID 0x172 through 013ce130 and 013ce230. |

**CustomChoiceDialogState** (17)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01476970` | CustomChoiceDialogState::~CustomChoiceDialogState | confirmed | Destructor body of CustomChoiceDialogState: stores CustomChoiceDialogState::vftable into *this, destroys three members through 01300cd0 (called three times), t… |
| `01476a20` | CustomChoiceDialogState::GetTypeName | confirmed | Virtual slot 11 of CustomChoiceDialogState. Returns the constant string 'CustomChoiceDialogState' (mov eax, imm; ret). |
| `01476a60` | CustomChoiceDialogState::ButtonPressed | confirmed | Virtual slot 40 of CustomChoiceDialogState. Logs 'CustomChoiceDialogState::button pressed, player %d, button %d'. When the player id equals this+0xf8 it builds… |
| `01476d00` | CustomChoiceDialogState::Read | confirmed | Virtual slot 9 of CustomChoiceDialogState. Calls vtable+0x10 (slot 4, header read) and returns 0 on failure; then base read 0147c850; then 013c24d0 into this+0… |
| `01476e10` | CustomChoiceDialogState::Write | confirmed | Virtual slot 10 of CustomChoiceDialogState. Output argument param_2: zero calls vtable+0x4 (slot 1) with no real argument, otherwise vtable+0x8 (slot 2, header… |
| `01476f50` | CustomChoiceDialogState::DeletingDtor | confirmed | Virtual slot 0 of CustomChoiceDialogState: the scalar deleting destructor. Calls 01476970 (the CustomChoiceDialogState destructor body), frees the object with … |
| `01477160` | CustomChoiceDialogState::stateSpecificMessage | confirmed | Virtual slot [42] of CustomChoiceDialogState. Builds trace text 'CustomChoicedialogState::stateSpecificMessage %d, %s, %s' (player id param_2 and two std::stri… |
| `01477430` | CustomChoiceDialogState::buttonPressed | confirmed | Virtual slot [41] of CustomChoiceDialogState. Copies the three button label std::strings at this+0x8C, +0xA8, +0xC4 and lowercases each with 012a26c0 (_tolower… |
| `01477c80` | CustomChoiceDialogState::CustomChoiceDialogState | confirmed | Default constructor (__fastcall, returns this). Stores CustomChoiceDialogState::vftable at this, calls base ctor 0147c980 and three 012fa910 member-initialisat… |
| `01477d40` | CustomChoiceDialogState::CustomChoiceDialogState | confirmed | Constructor taking one pointer argument (param_2). Calls base ctor 0147c9f0 with param_2, stores the vtable CustomChoiceDialogState::vftable, runs three 012fa9… |
| `01477ea0` | CustomChoiceDialogState::checkActionPrecon | confirmed | Thiscall (ECX=this). Computes two booleans by ANDing a 012faa80 check with a vtable +0x68 predicate applied to each entry of a vector (loop over local_d0..loca… |
| `01478410` | CustomChoiceDialogState::updateState | confirmed | Virtual slot [15] (+0x3c) of CustomChoiceDialogState; the other handlers call it after setting this+0x124. Logs 'ChoiceChoiceDialogState::updateState %d' with … |
| `01478ce0` | CustomChoiceDialogState::setupPhase | confirmed | Virtual slot [14] (+0x38) of CustomChoiceDialogState. Reads the returnValueMap: asserts hasIntegerKey(kPlayerReturn) (key 4) and stores the player id at this+0… |
| `01476a30` | CustomChoiceDialogState::GetStaticDword | likely | Virtual slot 12 of CustomChoiceDialogState. Returns the global dword DAT_01cbe5a0 (mov eax,[abs]; ret), shared by all instances of the class. |
| `01476a40` | CustomChoiceDialogState::SetStaticDword | likely | Virtual slot 13 of CustomChoiceDialogState. Loads its stack argument and stores it in the global dword DAT_01cbe5a0, then returns with ret 4 (one dword argumen… |
| `01476b60` | CustomChoiceDialogState::SubmitChoiceRecords | likely | Virtual slot 18 of CustomChoiceDialogState. If 013836c0 is false, it still proceeds only when 013837f0() is nonzero; in that case it builds a 0x18-byte record … |
| `01476f70` | CustomChoiceDialogState::UpdateState | likely | Not a vtable slot. Logs 'CustomChoiceDialogState::updateState - do effect %d' with this+0x120 (the button or effect index stored by ButtonPressed), then runs t… |

**DeckBuilderUtilities** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012db270` | DeckBuilderUtilities::~DeckBuilderUtilities | confirmed | Stores DeckBuilderUtilities::vftable into *this, clears DAT_01cbd6a8 when this is the singleton, frees the vector members at this+0x50, +0x40, +0x24 and +0x8 (… |
| `012dbb30` | DeckBuilderUtilities::DeckBuilderUtilities | confirmed | Stores DeckBuilderUtilities::vftable into *this, zeroes the vector triples at this+8, +0x24, +0x40 and +0x50, and constructs two member sub-objects through 012… |
| `012dbbd0` | DeckBuilderUtilities::DestroyMaybeFree | likely | Virtual slot 12 of DeckBuilderUtilities::vftable: calls the destructor body 012db270 on this, then frees this with _free when bit 0 of the flag argument is set… |
| `012dd810` | DeckBuilderUtilities::ResolveCollectionAndForward | likely | Virtual slot 2 shared by DeckBuilderUtilities::vftable and CWDeckBuilderUtilities::vftable: resolves the collection id (param_4) through 012ae1a0; when it is m… |

**DeckCommand_PopulateOnlineDeckData** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e4150` | DeckCommand_PopulateOnlineDeckData::Write | confirmed | Thiscall virtual (slot 10). Calls virtual slot 2 (vtable +8) with the buffer; calls 013c26e0 with "Starting DeckCommand_PopulateOnlineDeckData " (no-op, lead-v… |
| `014e4410` | DeckCommand_PopulateOnlineDeckData::~DeckCommand_PopulateOn… | confirmed | Fastcall destructor body (non-deleting). Stores the PopulateOnlineDeckData vftable into *this; for each non-null pointer in the vector at +8..+0xc calls its sc… |
| `014e4560` | DeckCommand_PopulateOnlineDeckData::GetTypeName | confirmed | Returns the address of the constant string "DeckCommand_PopulateOnlineDeckData" (mov eax, imm32; ret). |
| `014e45d0` | DeckCommand_PopulateOnlineDeckData::DeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Calls the destructor body 014e4410 on this, then if bit 0 of the flag argument is set calls _free on this; returns … |
| `014e4660` | DeckCommand_PopulateOnlineDeckData::DebugPrint | confirmed | Thiscall virtual (slot 19). Calls 01241650 with "Populate Deck Online Data Deck Command " (len 0x27), then base text routine 014424d0(param_2). Then for each e… |
| `014e4760` | DeckCommand_PopulateOnlineDeckData::PrintTag | confirmed | Virtual (slot 20). Calls 01241650 with the fixed text "Populate Online Deck Data Deck Command" (len 0x26) and returns nothing. |
| `014e4ac0` | DeckCommand_PopulateOnlineDeckData::DeckCommand_PopulateOnl… | confirmed | Fastcall default constructor. Calls base init 01442180 with this (its argument is the debug/cookie value), stores the PopulateOnlineDeckData vftable at +0, zer… |
| `014e4c90` | DeckCommand_PopulateOnlineDeckData::CloneInto | confirmed | Thiscall virtual (slot 18, argument = destination NetworkCommand*). Dynamic-casts the argument to PopulateOnlineDeckData (00d8d382); if null calls 012f5a60 wit… |
| `014e4db0` | DeckCommand_PopulateOnlineDeckData::Read | confirmed | Thiscall virtual (slot 9 override). Calls virtual slot 4 (vtable +0x10) with (param_2, &local); if nonzero calls base read 01442360(param_2); if nonzero reads … |
| `014e4570` | DeckCommand_PopulateOnlineDeckData::GetStaticValue | likely | Returns the 32-bit global DAT_01cbe8c8 (mov eax,[01cbe8c8]; ret). No this pointer is read. |
| `014e4580` | DeckCommand_PopulateOnlineDeckData::SetStaticValue | likely | Stores its one stack argument into the global DAT_01cbe8c8 (mov [01cbe8c8],eax; ret 4). No this pointer is read. |
| `014e4b20` | DeckCommand_PopulateOnlineDeckData::Clone | likely | Fastcall virtual (slot 17). Allocates 0x14 bytes with operator new (00d833a6); if non-null constructs the new object with 014e4ac0 (default ctor, no argument) … |
| `014e4c00` | DeckCommand_PopulateOnlineDeckData::Create | likely | Non-member function. Allocates 0x14 bytes with operator new (00d833a6); if non-null calls 014e4ac0 (default ctor) with the block in ECX and returns its EAX; if… |

**DiscardCardState** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01475520` | DiscardCardState::SynchPointReached | confirmed | Virtual slot 44 of DiscardCardState. Logs 'DiscardCardState::synchPointReached'. If this+0x104 is positive it increments this+0x114. When this+0x110 is nonzero… |
| `014755a0` | DiscardCardState::LeaveState | confirmed | Virtual slot 18 of DiscardCardState. Logs 'DiscardCardState::leaveState', calls 013f39c0, allocates a 0x18-byte record (00d833a6, ctor 01417e60), sets record k… |
| `01475640` | DiscardCardState::~DiscardCardState | confirmed | Destructor body of DiscardCardState: stores DiscardCardState::vftable into *this, frees the buffer at this+0x11c when set, zeroes this+0x11c, +0x120 and +0x124… |
| `014756e0` | DiscardCardState::GetTypeName | confirmed | Virtual slot 11 of DiscardCardState. Returns the constant string 'DiscardCardState'. |
| `01475720` | DiscardCardState::Write | confirmed | Virtual slot 10 of DiscardCardState. Output argument param_2: when zero it calls vtable+0x4 (slot 1) with no real argument, otherwise vtable+0x8 (slot 2, heade… |
| `01475870` | DiscardCardState::DeletingDtor | confirmed | Virtual slot 0 of DiscardCardState: the scalar deleting destructor. Calls 01475640 (the DiscardCardState destructor body), frees the object with _free when bit… |
| `01475890` | DiscardCardState::DiscardCardState | confirmed | Default constructor: calls 0147c980 (base constructor), stores DiscardCardState::vftable, zeroes this+0x11c, +0x120 and +0x124, assigns 'DiscardCardState' (16 … |
| `01475950` | DiscardCardState::CtorFromMachine | confirmed | Constructor taking param_2 (the state machine), passed to 0147c9f0 (base constructor). Then the same field initialisation as DiscardCardState::DiscardCardState… |
| `014765d0` | DiscardCardState::RestoreState | confirmed | Virtual slot 17 of DiscardCardState. Logs 'DiscardCardState::restoreState (%d, %d)' with this+0x114 and this+0x104. If this+0x104 <= this+0x114 it returns 1. O… |
| `01476630` | DiscardCardState::Read | confirmed | Virtual slot 9 of DiscardCardState. Calls vtable+0x10 (slot 4, header read) and returns 0 when it fails; then base read 0147c850, then serial integer reads (01… |
| `01476760` | DiscardCardState::EnterState | confirmed | Virtual slot 14 of DiscardCardState. Logs 'DiscardCardState::enterState', calls this vtable+0x4c, then reads the state's return-value map (013f37e0 and 0141738… |
| `014756f0` | DiscardCardState::GetStaticDword | likely | Virtual slot 12 of DiscardCardState. Returns the global dword DAT_01cbe59c (shared by all instances of the class). |
| `01475700` | DiscardCardState::SetStaticDword | likely | Virtual slot 13 of DiscardCardState. Stores its argument into the global dword DAT_01cbe59c and returns. |
| `01475a10` | DiscardCardState::Factory | likely | Allocates 0x128 bytes (00d833a6, operator new), calls DiscardCardState::DiscardCardState (01475890) on the block and returns it; returns 0 when the allocation … |
| `01475aa0` | DiscardCardState::SelectAndSubmitDiscard | likely | Not a vtable slot; called from DiscardCardState::EnterState (01476760) and 014765d0. If the candidate list at this+0x11c is empty it selects the candidates: it… |

**DivideExpression** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0149cb30` | DivideExpression::DivideExpression | confirmed | Constructor body: runs the Expression base constructor 0149adc0 and stores the DivideExpression vtable (018f9d64). |
| `0149cbb0` | DivideExpression::GetTypeName | confirmed | Virtual slot [11] of DivideExpression. Returns the literal 'DivideExpression'. |
| `0149cc40` | DivideExpression::AssertFalseStub | confirmed | Virtual slot [15] of DivideExpression. Calls the assert helper 012f5a60 with 'false', source DivideExpression.cpp and line 0x12, then returns 0. |
| `0149cc60` | DivideExpression::Evaluate | confirmed | Virtual slot [14] of DivideExpression. Traces '(Divide'. Evaluates the lhs and rhs children through slot [14]; both must be integers. Reads integer payloads wi… |
| `0149ce70` | DivideExpression::ScalarDeletingDtor | confirmed | Virtual slot [0]: stores the DivideExpression vtable, calls the base destructor 0149ae80, frees the object with _free when bit 0 of the argument is set. Return… |
| `0149cdf0` | DivideExpression::Create | likely | Factory: operator new 00d833a6 with size 0x24; on null it returns 0, otherwise it constructs with 0149cb30 and returns the object. |
| `0149ce50` | DivideExpression::RegisterFactory | likely | Registers factory 0149cdf0 under classID 0x112 through 013ce130 and 013ce230. |

**DivideRoundExpression** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0149c660` | DivideRoundExpression::DivideRoundExpression | confirmed | Constructor body: runs the Expression base constructor 0149adc0, stores the DivideRoundExpression vtable (018f9c34), and sets the byte at +0x24 to 1 (round-up … |
| `0149c6e0` | DivideRoundExpression::GetTypeName | confirmed | Virtual slot [11] of DivideRoundExpression. Returns the literal 'DivideRoundExpression'. |
| `0149c770` | DivideRoundExpression::AssertFalseStub | confirmed | Virtual slot [15] of DivideRoundExpression. Calls the assert helper 012f5a60 with 'false', source DivideRoundExpression.cpp and line 0x15, then returns 0. |
| `0149cad0` | DivideRoundExpression::ScalarDeletingDtor | confirmed | Virtual slot [0]: stores the DivideRoundExpression vtable, calls the base destructor 0149ae80, frees the object with _free when bit 0 of the argument is set. R… |
| `0149ca50` | DivideRoundExpression::Create | likely | Factory: operator new 00d833a6 with size 0x28; on null it returns 0, otherwise it constructs with 0149c660 and returns the object. |
| `0149cab0` | DivideRoundExpression::RegisterFactory | likely | Registers factory 0149ca50 under classID 0x15d through 013ce130 and 013ce230. |

**DoublePassState** (20)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01472710` | DoublePassState::RestoreState | confirmed | Virtual slot 17 of DoublePassState (also CWDoublePassState slot 17). Logs 'DOUBLEPASS::RESTORESTATE'. If this+0xf4 is nonzero it dynamic-casts the machine (thi… |
| `01472a00` | DoublePassState::ProcessDoublePassEnd | confirmed | Virtual slot 78 of DoublePassState (also CWDoublePassState slot 78). Logs 'DOUBLEPASS::PROCESSDOUBLEPASSEND' and switches on this+0x120. Values 1 or 2 return 1… |
| `01472b80` | DoublePassState::LeaveState | confirmed | Virtual slot 18 of DoublePassState (also CWDoublePassState slot 18). Logs 'DOUBLEPASS::LEAVESTATE', calls 013f39c0, 01347e10 and 013f8020. When 013f8020 return… |
| `01472cc0` | DoublePassState::ButtonPressed | confirmed | Virtual slot 40 of DoublePassState (also CWDoublePassState slot 40). Logs 'DOUBLEPASS::BUTTONPRESSED player: %d, button: %d'. If this+0x128 is 3 and button is … |
| `01472f60` | DoublePassState::ButtonPressedWithString | confirmed | Virtual slot 41 of DoublePassState (also CWDoublePassState slot 41). Logs 'DOUBLEPASS::BUTTONPRESSEDWITHSTRING player: %d, button: %d, buttonText: %s'. Outside… |
| `014734e0` | DoublePassState::PassPushed | confirmed | Virtual slot 32 of DoublePassState (also CWDoublePassState slot 32). Logs 'DOUBLEPASS::PASSPUSHED playerid: %d'. If the player id equals the current player (01… |
| `01474040` | DoublePassState::RestoreStatePartTwo | confirmed | Virtual slot 73 of DoublePassState. Logs 'DOUBLEPASS::RESTORESTATEPARTTWO'. Calls 013f37e0 (saved as a pointer-sized value) and requires it nonzero, plus 01417… |
| `01474190` | DoublePassState::DoDoublePassLoop | confirmed | Virtual slot 76 of DoublePassState (also CWDoublePassState slot 76). Logs 'DOUBLEPASS::DODOUBLEPASSLOOP', stores param_2 (byte) at this+0x11c and param_3 at th… |
| `014745d0` | DoublePassState::StateSpecificMessage | confirmed | Virtual slot 42 of DoublePassState (also CWDoublePassState slot 42). Takes param_2 (player id) and two ValueData pointers param_3 and param_4; each yields an i… |
| `01474910` | DoublePassState::DoDoublePassLoopCompletion | confirmed | Virtual slot 77 of DoublePassState (also CWDoublePassState slot 77). Logs 'DOUBLEPASS::DODOUBLEPASSLOOPCOMPLETION', asserts this+0x108 (mCurrentPlayer, line 0x… |
| `01474c60` | DoublePassState::~DoublePassState | confirmed | Destructor body of DoublePassState: stores DoublePassState::vftable into *this, frees the buffer at this+0x110 through _free when set, zeroes this+0x110, +0x11… |
| `01474d10` | DoublePassState::GetTypeName | confirmed | Virtual slot 11 of DoublePassState. Returns the constant string 'DoublePassState'. |
| `01474d50` | DoublePassState::Read | confirmed | Virtual slot 9 of DoublePassState. Calls vtable+0x10 (slot 4, header read) with param_2 and bails out with 0 if it fails. Then calls base read 0147c850(param_2… |
| `01474fa0` | DoublePassState::Write | confirmed | Virtual slot 10 of DoublePassState. Takes param_2 as the output: when it is zero it calls vtable+0x4 (slot 1) with no real argument; otherwise it calls vtable+… |
| `01475230` | DoublePassState::DoublePassState | confirmed | Default constructor body: calls 0147c980 (base constructor), stores DoublePassState::vftable into *this, calls 0129d6c0, zeroes this+0x110, +0x114 and +0x118, … |
| `014752e0` | DoublePassState::DeletingDtor | confirmed | Virtual slot 0 of DoublePassState: the scalar deleting destructor. Calls 01474c60 (the DoublePassState destructor body), then frees the object with _free when … |
| `01475300` | DoublePassState::CtorFromMachine | confirmed | Constructor taking a state machine (param_2, passed to base constructor 0147c9f0, which binds the machine at +8 as read by 0145c4d0). Stores DoublePassState::v… |
| `01474d20` | DoublePassState::GetStaticDword | likely | Virtual slot 12 of DoublePassState. Returns the global dword DAT_01cbe598 (shared by all instances of the class). |
| `01474d30` | DoublePassState::SetStaticDword | likely | Virtual slot 13 of DoublePassState. Stores its argument into the global dword DAT_01cbe598 and returns. |
| `01475490` | DoublePassState::Factory | likely | Allocates 300 bytes (00d833a6, operator new), then calls the default constructor 01475230 on the block and returns it; returns 0 when the allocation is null. |

**DrawCardState** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014713a0` | DrawCardState::~DrawCardState | confirmed | Destructor body: stores DrawCardState::vftable into *this, calls 01300cd0, then 0147c520 (base cleanup); no delete. Called from four EH unwind funclets. |
| `01471480` | DrawCardState::enterState | confirmed | Logs the trace literal; calls vtable +0x4c with no argument; requires returnValueMap (013f37e0) and reads integer keys kPlayerReturn (4), kCardPlayedReturn (3)… |
| `014715d0` | DrawCardState::restoreState | confirmed | Logs the trace literal 'DrawCardState::restoreState' and calls vtable +0x124 with no argument. |
| `014718c0` | DrawCardState::DeletingDtor | confirmed | Scalar deleting destructor: calls the dtor body 014713a0; frees this through _free when bit 0 of the flag argument is set; returns this. |
| `014718e0` | DrawCardState::stateSpecificMessage | confirmed | Asserts arg1Value and arg2Value are non-null; logs 'DrawCardState::stateSpecificMessage %d, %s, %s' with the player id and two strings; checks player == this+0… |
| `01471b20` | DrawCardState::DrawCardState | confirmed | Constructor: runs 0147c980 on the base, stores DrawCardState::vftable, runs 012fa910, assigns the 13-byte literal 'DrawCardState' to the member string at this+… |
| `01471bd0` | DrawCardState::DrawCardState_Arg | confirmed | Constructor variant with one argument passed to 0147c9f0; stores DrawCardState::vftable, runs 012fa910, assigns the literal 'DrawCardState' to this+0x1c and ze… |
| `01471c80` | NewDrawCardState | likely | Allocates 0x128 bytes with 00d833a6; if non-null it constructs a DrawCardState through 01471b20 and returns it, else returns 0. |

**EachExpression** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0149ba60` | EachExpression::EachExpression | confirmed | Constructor body: runs the Expression base constructor 0149adc0, stores the EachExpression vtable (018f9a1c), and zeroes the dwords at +0x24 (card-list target)… |
| `0149bae0` | EachExpression::GetTypeName | confirmed | Virtual slot [11] of EachExpression. Returns the literal 'EachExpression'. |
| `0149bb90` | EachExpression::GetValidTargets | confirmed | Virtual slot [15] of EachExpression (also CWEachExpression [15]). Traces '(gvt: each' and 'executing each node....', then calls this object's own virtual slot … |
| `0149bdc0` | EachExpression::ScalarDeletingDtor | confirmed | Virtual slot [0]: stores the EachExpression vtable, calls the base destructor 0149ae80, frees the object with _free when bit 0 of the argument is set. Returns … |
| `0149be20` | EachExpression::Evaluate | confirmed | Virtual slot [14] of EachExpression (also CWEachExpression [14]). Traces '(Each' and 'initialcards:' with the list from this slot [20] (0149c170). For each car… |
| `0149bd40` | EachExpression::Create | likely | Factory: operator new 00d833a6 with size 0x2c; on null it returns 0, otherwise it constructs with 0149ba60 and returns the object. |
| `0149bda0` | EachExpression::RegisterFactory | likely | Registers factory 0149bd40 under classID 0x23 through 013ce130 and 013ce230. |

**EmitTextMessageNode** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0149b460` | EmitTextMessageNode::EmitTextMessageNode | confirmed | Constructor body: runs the ExpressionTreeNode constructor 0149ab30, stores the EmitTextMessageNode vtable (018f992c), zeroes the optional child at +0x10 and se… |
| `0149b4e0` | EmitTextMessageNode::GetTypeName | confirmed | Virtual slot [11] of EmitTextMessageNode. Returns the literal 'EmitTextMessageNode'. |
| `0149b590` | EmitTextMessageNode::GetValidTargets | confirmed | Virtual slot [15] of EmitTextMessageNode. Traces 'gvt:EmitTextMessageNode' and ') => false', then returns 0 with no candidates. |
| `0149b720` | EmitTextMessageNode::ScalarDeletingDtor | confirmed | Virtual slot [0]: calls the destructor body 0149b520, then frees the object with _free when bit 0 of the argument is set. Returns this. |
| `0149b520` | EmitTextMessageNode::~EmitTextMessageNode | likely | Destructor body: stores the EmitTextMessageNode vtable; if the child at +0x10 is non-null it calls that child's slot 0 with flag 1 and clears it; then runs the… |
| `0149b6a0` | EmitTextMessageNode::Create | likely | Factory: operator new 00d833a6 with size 0x18; on null it returns 0, otherwise it constructs with 0149b460 and returns the object. |
| `0149b700` | EmitTextMessageNode::RegisterFactory | likely | Registers factory 0149b6a0 under classID 0x30 through 013ce130 and 013ce230. |

**Engine::CardRef** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0140c010` | Engine::CardRef::ScalarDeletingDtor | confirmed | Stores the Engine::CardRef vtable, calls base teardown 01514fc0, frees the object when bit 0 of the stack argument is set, and returns this (ret 4). |
| `0140bf60` | Engine::CardRef::Ctor | likely | Calls 01514f60 (base constructor of the FRTW card-reference family) and stores the Engine::CardRef vtable into *param_1; returns param_1. |
| `0140bfc0` | Engine::CardRef::DtorBody | likely | Stores the Engine::CardRef vtable into *param_1 and calls base teardown 01514fc0; releases no member storage in this body. |

**Engine::FRTWAddModel** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01258470` | Engine::FRTWAddModel::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWAddModel::vftable into *this; if the argument is nonzero calls 012ea4f0 (optional init). Re… |
| `012584e0` | Engine::FRTWAddModel::GetActionName | likely | Returns the constant string pointer to the literal AddModel (mov eax, imm32; ret). Vtable slot [1] of Engine::FRTWAddModel::vftable. |
| `01258500` | Engine::FRTWAddModel::GetTraceLabel | likely | Returns the constant string pointer to the trace label AddModel : 0 : with the frtwgamehandlers.cc:141 source location (mov eax, imm32; ret). Vtable slot [4] o… |
| `01258510` | Engine::FRTWAddModel::ScalarDeletingDtor | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this; when bit 0 of the argument is set calls _free on this (00d82463 thunk). Returns this. |

**Engine::FRTWArchetypeView** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01515ec0` | Engine::FRTWArchetypeView::~FRTWArchetypeView | confirmed | Stores Engine::FRTWArchetypeView::vftable into *this, calls FUN_0143a8b0 (base destructor), and returns; called from EH unwind funclets and two other callers. |
| `01515f10` | Engine::FRTWArchetypeView::ScalarDeletingDtor | confirmed | Virtual slot 0 of Engine::FRTWArchetypeView. Stores the vtable, calls FUN_0143a8b0, then calls _free on this when bit 0 of the second argument is set. Returns … |
| `01515f70` | Engine::FRTWArchetypeView::FRTWArchetypeView | likely | Calls FUN_0143a850 on this, stores Engine::FRTWArchetypeView::vftable into *this, then pushes 4-byte ids into the list at this+4 through FUN_0042c155 in order:… |

**Engine::FRTWAttackTargetFromSquare** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01259760` | Engine::FRTWAttackTargetFromSquare::Slot2 | likely | Virtual slot [2] of Engine::FRTWAttackTargetFromSquare::vftable. Looks up the arguments named Target and Square through 012eb290; calls 012e0ba0 and 0124d010; … |

**Engine::FRTWCardRef** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01514fc0` | Engine::FRTWCardRef::~FRTWCardRef | confirmed | Stores Engine::FRTWCardRef::vftable into *this, calls FUN_01380920 (base destructor), and returns; called from EH unwind funclets. |
| `01515100` | Engine::FRTWCardRef::ScalarDeletingDtor | confirmed | Virtual slot 0 of Engine::FRTWCardRef. Stores the vtable, calls FUN_01380920 (base destructor), then calls _free on this when bit 0 of the second argument is s… |
| `01514f60` | Engine::FRTWCardRef::FRTWCardRef | likely | Calls FUN_013808b0 on this, then stores Engine::FRTWCardRef::vftable into *this and returns this. |

**Engine::FRTWClearWaitingDialog** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0125a190` | Engine::FRTWClearWaitingDialog::Slot2 | likely | Virtual slot [2] of Engine::FRTWClearWaitingDialog::vftable. Builds the Scaleform action CloseOpponentWaitingDialog through 012ec220, then dispatches it throug… |

**Engine::FRTWDeckService** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013a9f20` | Engine::FRTWDeckService::FRTWDeckService | confirmed | Constructor: calls base ctor 013de860 first, stores Engine::FRTWDeckService::vftable into *this, sets +0x118 and +0x11c to -1, then issues a series of calls wi… |
| `013aa090` | Engine::FRTWDeckService::DtorBody | confirmed | Stores Engine::FRTWDeckService::vftable into *this, then calls the base dtor body 013de920 (Engine::DeckServiceBase) on this. Frees nothing. |
| `013aa1c0` | Engine::FRTWDeckService::ScalarDeletingDtor | confirmed | Stores the FRTWDeckService vftable into *this, calls base dtor body 013de920 on this, then calls _free(this) when bit 0 of its byte argument is set; returns th… |

**Engine::FRTWEngineConsiderPlayer** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01259580` | Engine::FRTWEngineConsiderPlayer::Slot2 | likely | Virtual slot [2] of Engine::FRTWEngineConsiderPlayer::vftable. Calls 012e0ba0 and 0124d010; when nonzero reads argument 1 through 012f8f60(1), checks its type … |

**Engine::FRTWGameService** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01321680` | Engine::FRTWGameService::~FRTWGameService | confirmed | Fastcall on this. Stores Engine::FRTWGameService::vftable at *this, calls 01321610 (StdTreeTidyFreeHead) for three tree members, frees the buffers at this+0x78… |
| `013217f0` | Engine::FRTWGameService::ctor | confirmed | Fastcall, returns this. Calls 0131c8a0 for the base, stores Engine::FRTWGameService::vftable at *this, sets flag/state fields (+0x30 byte 1, +0x34 and +0x38 = … |
| `01321900` | Engine::FRTWGameService::ScalarDeletingDtor | confirmed | Thiscall with a flag byte argument (ret 4). Calls 01321680 (the FRTWGameService destructor body), then frees this through _free when bit 0 of the flag is set, … |

**Engine::FRTWRequestSync** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01258570` | Engine::FRTWRequestSync::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable then Engine::FRTWRequestSync::vftable into *this; if the argument is nonzero calls 012ea300 (optional init). Retu… |
| `01258600` | Engine::FRTWRequestSync::GetTraceLabel | likely | Returns the constant string pointer to the trace label kRequestSyncDisplayAction : 0 : with the frtwgamehandlers.cc:323 source location (mov eax, imm32; ret). … |
| `01258610` | Engine::FRTWRequestSync::ScalarDeletingDtor | likely | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the argument is set calls _free on this (00d82463 thunk). Returns this. |

**Engine::FRTWSpawnWaitingDialog** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0125a0d0` | Engine::FRTWSpawnWaitingDialog::Slot2 | likely | Virtual slot [2] of Engine::FRTWSpawnWaitingDialog::vftable. Builds the Scaleform action OpenOpponentWaitingDialog through 012ec220, sets DialogText to 'Your o… |

**Engine::GameService** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01320950` | Engine::GameService::PushPlayerPoints | likely | Virtual slot [6]. Calls vtable +0x14c on the member at this+0xc to fill a vector of player objects and iterates it. For each player it calls this->vtable+0x48 … |
| `01320ba0` | Engine::GameService::AddModelEvent | likely | Virtual slot [18] of Engine::GameService::vftable, shared with Engine::FRTWGameService. Returns 0 when FUN_0131c610(this) fails. Otherwise it reads property 0x… |

**EqualToExpression** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0149b110` | EqualToExpression::EqualToExpression | confirmed | Constructor body: runs base constructor 0149adc0 and stores the EqualToExpression vtable (018f988c). |
| `0149b190` | EqualToExpression::GetTypeName | confirmed | Virtual slot [11] of EqualToExpression. Returns the literal 'EqualToExpression'. |
| `0149b220` | EqualToExpression::AssertFalseStub | confirmed | Virtual slot [15] of EqualToExpression. Calls the assert helper 012f5a60 with 'false', source EqualToExpression.cpp and line 0x19, then returns 0. |
| `0149b240` | EqualToExpression::Evaluate | confirmed | Virtual slot [14] of EqualToExpression. Traces '(equals '. Evaluates lhs (+0x1c) and rhs (+0x20) through slot [14]; a failed child returns 0 after tracing ') =… |
| `0149b400` | EqualToExpression::ScalarDeletingDtor | confirmed | Virtual slot [0]: stores the EqualToExpression vtable, calls base destructor 0149ae80, frees the object with _free when bit 0 of the argument is set. Returns t… |
| `0149b1d0` | EqualToExpression::~EqualToExpression | likely | Destructor body without free: stores the EqualToExpression vtable, then calls base destructor 0149ae80 with the EH cookie. |
| `0149b380` | EqualToExpression::Create | likely | Factory: operator new 00d833a6 with size 0x24; on null it returns 0, otherwise it constructs with 0149b110 and returns the object. |
| `0149b3e0` | EqualToExpression::RegisterFactory | likely | Registers factory 0149b380 under classID 0x1f through 013ce130 and 013ce230. |

**EvaluationEnvironment** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013862d0` | EvaluationEnvironment::AddCommandObject | confirmed | Thiscall (ECX = environment) with a command object (param_2, asserted non-null at line 0x60c). It reads the object's name through 0142a7e0 and its id through 0… |
| `01386420` | EvaluationEnvironment::Write | confirmed | Virtual slot [10] of EvaluationEnvironment::vftable (thiscall). Calls the object's vtable slot 2 on the serial buffer first (base header), then writes integers… |
| `01386b70` | EvaluationEnvironment::CopyFrom | confirmed | Thiscall: asserts the source argument (named environment, EvaluationEnvironment.cpp line 0x55). Copies dwords +4, +8, +0xc, +0x10 and +0x14 from the source, co… |
| `01386c20` | EvaluationEnvironment::CopyValueDataToKey | confirmed | Thiscall with two stack arguments (key, source ValueData pointer). Asserts the source is non-null (named value, EvaluationEnvironment.cpp line 0xb5). Gets the … |
| `01388380` | EvaluationEnvironment::GetTargetWithEnterState | confirmed | Fastcall (ECX = environment). Returns the cached Target1 object (this+0xc through 01357a00) when set. Otherwise it sets key 0x1c to 1, and when the flag from 0… |
| `01385d90` | EvaluationEnvironment::GetCardsFromKey0xe | likely | Thiscall on an environment: asserts this+4 (mGame) is non-null (EvaluationEnvironment.cpp line 0x318), probes key 0xe with 013833c0 and requires the stored Val… |
| `01385f30` | EvaluationEnvironment::GetCardsFromKey0xc | likely | Same body as 01385d90 with key 0xc: asserts this+4 (mGame) non-null (line 0x391), probes key 0xc with 013833c0, requires an integer-list ValueData (type 6), an… |
| `01386ae0` | EvaluationEnvironment::Create | likely | Allocates 0x94 bytes with 00d833a6 (operator new). If the allocation is non-null it calls 01385940 on it and returns the result; otherwise it returns 0. |
| `013888d0` | EvaluationEnvironment::GetTarget1 | likely | Thiscall with one argument (param_2). Returns the Target1 object (this+0xc) when set. Otherwise it sets key 0x1c to 1 and, when the flag from 01383520(4) is cl… |
| `013892e0` | EvaluationEnvironment::GetTarget2Alt | likely | Thiscall with one argument. Same structure as 01388dc0 for the Target2 slot (this+0x10, list at +0x50, key 0x1c set to 1, mGame hook when the flag from 0138352… |

**ExecuteRuleState** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01470cf0` | ExecuteRuleState::~ExecuteRuleState | confirmed | Destructor body: stores ExecuteRuleState::vftable into *this, sets the cookie slot and calls 0147c520 (base cleanup); no delete. |
| `01470dc0` | ExecuteRuleState::enterState | confirmed | Logs 'Entering ExecuteRuleState'; calls vtable +0x4c with no argument; calls 01347ab0, and when it returns true logs 'Game is Over! bail.' and returns 2; other… |
| `01470e20` | ExecuteRuleState::restoreState | confirmed | Logs the trace literal 'ExecuteRuleState::restoreState'; returns 1 when the byte at this+0xfc is zero and 3 otherwise. |
| `01470e50` | ExecuteRuleState::leaveState | confirmed | Logs the trace literal 'ExecuteRuleState::leaveState', calls 013f39c0 with no argument and returns 1. |
| `01470fe0` | ExecuteRuleState::DeletingDtor | confirmed | Scalar deleting destructor: stores ExecuteRuleState::vftable, calls 0147c520, and frees this through _free when bit 0 of the flag argument is set; returns this. |
| `01471040` | ExecuteRuleState::updateState | confirmed | Logs 'ExecuteRuleState::updateState %d' with the id at this+0xf8; calls 0129fea0(id, version) to fetch a rule set; when that is null it logs 'Couldn't get Rule… |
| `01471220` | ExecuteRuleState::ExecuteRuleState | confirmed | Constructor: runs 0147c980 on the base, stores ExecuteRuleState::vftable, zeroes this+0xf8 and the byte at this+0xfc, and assigns the 16-byte literal 'ExecuteR… |
| `01471290` | ExecuteRuleState::ExecuteRuleState_Arg | confirmed | Constructor variant with one argument passed to 0147c9f0; stores ExecuteRuleState::vftable, zeroes this+0xf8 and the byte at +0xfc, and assigns the literal 'Ex… |
| `01471310` | NewExecuteRuleState | likely | Allocates 0x100 bytes with 00d833a6; if non-null it constructs an ExecuteRuleState through 01471220 and returns it, else returns 0. |

**Expression** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0149adc0` | Expression::Expression | confirmed | Constructor under an EH frame: calls the QueryNode constructor 01494290, stores the Expression vtable (018f97c4), and zeroes the two child pointers at +0x1c an… |
| `0149ae40` | Expression::GetTypeName | confirmed | Virtual slot [11] of Expression. Returns the literal 'Expression'. |
| `0149ae80` | Expression::~Expression | confirmed | Destructor body: stores the Expression vtable; if the child at +0x1c is non-null it calls that child's slot 0 with flag 1 and clears it; likewise for the child… |
| `0149af20` | Expression::AssertFalseStub | confirmed | Virtual slot [15] of Expression. Calls the assert helper 012f5a60 with 'false', source Expression.cpp and line 0x23, then returns 0. |
| `0149b090` | Expression::ScalarDeletingDtor | confirmed | Virtual slot [0]: calls the destructor body 0149ae80, then frees the object with _free when bit 0 of the argument is set. Returns this. |
| `0149b010` | Expression::Create | likely | Factory for the Expression base (classID 25): calls operator new 00d833a6 with size 0x24; on null returns 0, otherwise constructs with 0149adc0 and returns the… |
| `0149b070` | Expression::RegisterFactory | likely | Registers factory 0149b010 under classID 0x19 through 013ce130 and 013ce230. |

**ExpressionTreeNode** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0149a5c0` | ExpressionTreeNode::AssertFalseStub | confirmed | Virtual slot [15] of ExpressionTreeNode. Calls assert helper 012f5a60 with 'false', source ExpressionTreeNode.cpp and line 0x19, then returns 0. |
| `0149a5e0` | ExpressionTreeNode::AssertFalseStub | confirmed | Virtual slot [14] of ExpressionTreeNode. Calls assert helper 012f5a60 with 'false', source ExpressionTreeNode.cpp and line 0x21, then returns 0. |
| `0149a860` | ExpressionTreeNode::~ExpressionTreeNode | confirmed | Destructor body: stores the ExpressionTreeNode vtable; walks the NodeKeyValueMap at this+8 and for each entry calls the stored object's slot 0 with flag 1 (del… |
| `0149a9a0` | ExpressionTreeNode::GetTypeName | confirmed | Virtual slot [11] of ExpressionTreeNode. Returns the literal 'ExpressionTreeNode'. |
| `0149ab30` | ExpressionTreeNode::ExpressionTreeNode | confirmed | Constructor body: stores PersistentBase, then PersistentComponent, then ExpressionTreeNode vtables (018d100c, 018d1034, 018f975c), sets the flag local to 1 and… |
| `0149aba0` | ExpressionTreeNode::ScalarDeletingDtor | confirmed | Virtual slot [0]: calls the destructor body 0149a860, then frees the object with _free when bit 0 of the argument is set. Returns this. |
| `0149a680` | ExpressionTreeNode::WriteKeyValueMap | likely | Writes the std::map<int,ptr> at this+4 (its header at +4 of the map object, size at +8). Calls 013c2640 with the element count, walks the tree in order from th… |
| `0149aa90` | ExpressionTreeNode::ReadKeyValueMap | likely | Reads a count with 013c2690 and loops that many times: 014001c0 reads a key (param_2) and an object pointer, the pointer is dynamically cast to ValueData (00d8… |
| `0149ad40` | ExpressionTreeNode::Create | likely | Factory: operator new 00d833a6 with size 0x10; on null it returns 0, otherwise it constructs with 0149ab30 and returns the object. |
| `0149ada0` | ExpressionTreeNode::RegisterFactory | likely | Registers factory 0149ad40 under classID 0x27 through 013ce130 and 013ce230. |

**GameCommand_IntroduceCard** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0142ecb0` | GameCommand_IntroduceCard::Handle | confirmed | Asserts this+8 (mGame) and this+4 (mPlayerID) are nonzero (file GameCommandIntroduceCard.cpp, lines 0x2b and 0x2c), calls FUN_0141b020, looks up FUN_01371b10(t… |

**GameCommand_ReadyForStartOfGame** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0141a060` | GameCommand_ReadyForStartOfGame::GameCommand_ReadyForStartO… | confirmed | Fastcall with ECX = this. Runs the base constructor FUN_0141b5d0 under an SEH frame, stores GameCommand_ReadyForStartOfGame::vftable into this, restores the fr… |
| `0141a0e0` | GameCommand_ReadyForStartOfGame::GetTypeName | confirmed | Virtual slot 11 of GameCommand_ReadyForStartOfGame::vftable. Returns the address of the text GameCommand_ReadyForStartOfGame (mov eax, imm32; ret). |
| `0141a120` | GameCommand_ReadyForStartOfGame::ConstructWithArg | confirmed | Thiscall with one stack argument. Runs the base constructor FUN_0141b5d0 under an SEH frame, stores GameCommand_ReadyForStartOfGame::vftable into this, calls F… |
| `0141a200` | GameCommand_ReadyForStartOfGame::CreateCommandForGame | confirmed | Virtual slot 17 of GameCommand_ReadyForStartOfGame::vftable. Fastcall with ECX = this. Asserts mGame at GameCommandReadyForStartOfGame.cpp line 0x29 when this+… |
| `0141a320` | GameCommand_ReadyForStartOfGame::Serialize | confirmed | Virtual slot 10 of GameCommand_ReadyForStartOfGame::vftable. Calls the writer virtual at vtable+8 on this with the writer argument, writes the labels Starting … |
| `0141a3f0` | GameCommand_ReadyForStartOfGame::ScalarDeletingDtor | confirmed | Virtual slot 0 of GameCommand_ReadyForStartOfGame::vftable with the scalar deleting shape: stores the class vtable into this, calls FUN_0141b310 (base teardown… |
| `0141a0f0` | GameCommand_ReadyForStartOfGame::GetStaticDword | likely | Virtual slot 12 of GameCommand_ReadyForStartOfGame::vftable. Returns the dword in the global DAT_01cbe260 (mov eax,[abs]; ret). |
| `0141a100` | GameCommand_ReadyForStartOfGame::SetStaticDword | likely | Virtual slot 13 of GameCommand_ReadyForStartOfGame::vftable. Stores its one stack argument into the global DAT_01cbe260 and returns with ret 4. |
| `0141a190` | GameCommand_ReadyForStartOfGame::~GameCommand_ReadyForStart… | likely | Fastcall with ECX = this. Stores GameCommand_ReadyForStartOfGame::vftable into this first and then calls FUN_0141b310 with the object as receiver. Returns noth… |

**GameCommand_StateSpecificMessage** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0141a480` | GameCommand_StateSpecificMessage::GameCommand_StateSpecific… | confirmed | Fastcall with ECX = this. Runs the base constructor FUN_0141b5d0 under an SEH frame, stores GameCommand_StateSpecificMessage::vftable into this, zeroes the wor… |
| `0141a510` | GameCommand_StateSpecificMessage::GetTypeName | confirmed | Virtual slot 11 of GameCommand_StateSpecificMessage::vftable. Returns the address of the text GameCommand_StateSpecificMessage (mov eax, imm32; ret). |
| `0141a550` | GameCommand_StateSpecificMessage::ConstructFromSource | confirmed | Thiscall with one stack argument (the source). Runs the base constructor FUN_0141b5d0 under an SEH frame, stores GameCommand_StateSpecificMessage::vftable, zer… |
| `0141a890` | GameCommand_StateSpecificMessage::CreateCommandForGame | confirmed | Virtual slot 17 of GameCommand_StateSpecificMessage::vftable. Fastcall with ECX = this. Allocates 0x38 bytes and, when that succeeds, constructs a message with… |
| `0141a900` | GameCommand_StateSpecificMessage::CloneInto | confirmed | Virtual slot 18 of GameCommand_StateSpecificMessage::vftable. Thiscall with one stack argument (the target). Dynamic-casts the argument from NetworkCommand to … |
| `0141a970` | GameCommand_StateSpecificMessage::Serialize | confirmed | Virtual slot 10 of GameCommand_StateSpecificMessage::vftable. Calls the writer virtual at vtable+8 with the writer, writes the Starting and Ending labels throu… |
| `0141aa40` | GameCommand_StateSpecificMessage::ExecuteOnGame | confirmed | Virtual slot 15 of GameCommand_StateSpecificMessage::vftable. Fastcall with ECX = this. Asserts mGame at GameCommandStateSpecificMessage.cpp line 0xda when thi… |
| `0141abb0` | GameCommand_StateSpecificMessage::~GameCommand_StateSpecifi… | confirmed | Fastcall with ECX = this. Stores GameCommand_StateSpecificMessage::vftable into this, releases the sub-object at +0x30 through its vtable slot 0 with 1 and cle… |
| `0141a520` | GameCommand_StateSpecificMessage::GetStaticDword | likely | Virtual slot 12 of GameCommand_StateSpecificMessage::vftable. Returns the dword in the global DAT_01cbe264 (mov eax,[abs]; ret). |
| `0141a530` | GameCommand_StateSpecificMessage::SetStaticDword | likely | Virtual slot 13 of GameCommand_StateSpecificMessage::vftable. Stores its one stack argument into the global DAT_01cbe264 and returns with ret 4. |

**GameData** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01429220` | GameData::DtorBody | confirmed | Destructor body: stores GameData::vftable, then calls 01428070 (TreeTidyAndFreeHead_Isnil15) on one map, 01428000 (TreeTidyAndFreeHead_Isnil15, unwind-referenc… |
| `01429300` | GameData::GameData | confirmed | Constructor (fastcall): stores GameData::vftable, initializes the family-A map at this+4 (01429290), then initializes two family-B maps (01428140 and 014281b0)… |
| `01429450` | GameData::~GameData | confirmed | Scalar deleting destructor (vtable slot [0]): calls the destructor body 01429220 and frees the object when bit 0 of the flag is set, returning the object. |
| `01426dc0` | GameData::LowerBoundDiffersFromCached | likely | Bool predicate (vtable slot [4], one stack argument, ret 4): lower_bound (01426be0) of the int key pointed to by the argument in the map at this+4; asserts (00… |
| `01428b30` | GameData::EraseKeyIfNotCached | likely | Void slot [3] with one stack argument (a key pointer): lower_bound on the key in the map at this+4; asserts (00d83c2d) unless the owner is this+4; when the fou… |
| `01428f50` | GameData::UpdateStat | likely | Virtual slot [1] (thiscall, three stack args): lower_bound (01426c50) of the key in the map owned at this+0x10 (owner assert 00d83c2d); when the found node dif… |
| `014290f0` | GameData::GetStatValue | likely | Virtual slot [2] (two stack arguments): resolves the entry for the argument through 01428e20 and the search 004f9ebd (owner assert); when the search result equ… |

**GameTurn** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013fcba0` | GameTurn::~GameTurn | confirmed | Thiscall destructor body. Stores GameTurn::vftable, zeroes +0x18, frees the buffers at +0x5c and +0x70 (zeroing them), frees the std::string buffer at +0x90 wh… |
| `013fccd0` | GameTurn::TypeName | confirmed | Virtual at GameTurn::vftable[11]: returns the address of the literal 'GameTurn' (018e3b30) with no arguments. |
| `013fd410` | GameTurn::GameTurn | confirmed | Fastcall constructor: stores PersistentBase::vftable, then PersistentComponent::vftable, then GameTurn::vftable. Calls the tree default constructors 013fc3c0 (… |
| `013fd520` | GameTurn::ScalarDeletingDtor | confirmed | Virtual slot 0 of GameTurn::vftable: calls the GameTurn destructor body 013fcba0 on this, frees the object with _free when the flag argument has bit 0 set, and… |
| `013fcce0` | GameTurn::GetStaticDword | likely | Virtual at GameTurn::vftable[12]: returns the global dword DAT_01cbe09c (no arguments). |
| `013fccf0` | GameTurn::SetStaticDword | likely | Virtual at GameTurn::vftable[13]: stores its stack argument into the global dword DAT_01cbe09c (ret 4). |
| `013fdf00` | GameTurnFactory | likely | Allocates 0xa8 bytes with operator new (00d833a6); when non-null, constructs GameTurn in place through 013fd410 and returns the object, else returns 0. |
| `013fdf70` | GameTurnRegisterFactory | likely | Registers the GameTurn factory 013fdf00 under classID 0xe8 (232) with the registry calls 013ce130 and 013ce230. |

**GetSimultaneousTargetState** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0146f3c0` | GetSimultaneousTargetState::restoreState | confirmed | Logs the trace literal 'GetSimultaneousTargetState::restoreState' through 0145c4d0 and 013508b0, then calls receiver vtable +0x3c (slot 15, GetSimultaneousTarg… |
| `0146f8b0` | GetSimultaneousTargetState::buttonPressed | confirmed | Logs the trace literal with two int args; looks up a keyed slot via 004f9ebd; if it already equals this+0x108 (the keyed slot at this+0x104) it logs and return… |
| `0146fa20` | GetSimultaneousTargetState::buttonPressedWithString | confirmed | Same shape as buttonPressed with a fourth string argument: logs the trace with (%d, %d, %s), checks the keyed slot for a duplicate, checks force mode (01383870… |
| `0146fe30` | GetSimultaneousTargetState::updateState | confirmed | Logs the trace literal; calls 0146f3f0 and, when it returns false, calls 0146f520 and returns 2. Otherwise allocates a 0x18-byte object (00d833a6), initialises… |
| `014700b0` | GetSimultaneousTargetState::GetSimultaneousTargetState | confirmed | Constructor: runs 0147c980 and 01247380 on the base, stores GetSimultaneousTargetState::vftable into *this, runs 005258fb, then assigns the 26-byte literal 'Ge… |
| `01470140` | GetSimultaneousTargetState::DeletingDtor | confirmed | Scalar deleting destructor: calls the dtor body 0146ffd0; when bit 0 of the flag argument is set it frees this through _free; returns this. |
| `01470160` | GetSimultaneousTargetState::GetSimultaneousTargetState_Arg | confirmed | Constructor variant taking one extra argument: calls 0147c9f0 with it, stores GetSimultaneousTargetState::vftable into *this, runs 01247380 and 005258fb, and a… |
| `01470280` | GetSimultaneousTargetState::enterState | confirmed | Calls vtable +0x4c of this with no arguments, logs the enter trace, requires returnValueMap (013f37e0), reads the integer key kValidTargetsReturn through 01417… |
| `014701f0` | NewGetSimultaneousTargetState | likely | Allocates 0x110 bytes with 00d833a6; if the allocation is non-null it constructs a GetSimultaneousTargetState with 014700b0 and returns the object, else return… |

**GetTopCardState** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014704a0` | GetTopCardState::~GetTopCardState | confirmed | Destructor body: stores GetTopCardState::vftable into *this, sets the cookie slot and calls 0147c520 (base cleanup); no delete. |
| `01470550` | GetTopCardState::restoreState | confirmed | Logs the trace literal 'GetTopCardState::restoreState' through 0145c4d0 and 013508b0, then returns 2. |
| `01470570` | GetTopCardState::leaveState | confirmed | Logs the trace literal 'GetTopCardState::leaveState', calls 013f39c0 with no argument and returns 1. |
| `01470630` | GetTopCardState::DeletingDtor | confirmed | Scalar deleting destructor: stores GetTopCardState::vftable, calls 0147c520, and frees this through _free when bit 0 of the flag argument is set; returns this. |
| `01470690` | GetTopCardState::stateSpecificMessage | confirmed | Formats the trace 'GetTopCardState::stateSpecificMessage %d, %s, %s' from an int and two strings (012fe040 conversions); maps this+0xf8 through a switch (3->2,… |
| `01470890` | GetTopCardState::GetTopCardState | confirmed | Constructor: runs 0147c980 on the base, stores GetTopCardState::vftable into *this, zeroes the field at this+0xf8 and assigns the 15-byte literal 'GetTopCardSt… |
| `01470900` | GetTopCardState::GetTopCardState_Arg | confirmed | Constructor variant with one argument passed to 0147c9f0; stores GetTopCardState::vftable, zeroes this+0xf8 and assigns the literal 'GetTopCardState' to the me… |
| `01470a00` | GetTopCardState::enterState | confirmed | Logs the trace literal; calls vtable +0x4c with no argument; requires returnValueMap (013f37e0) and integer keys kPlayerReturn (4) and kPlayAreaReturn (0xf) vi… |
| `01470970` | NewGetTopCardState | likely | Allocates 0xfc bytes with 00d833a6; if non-null it constructs a GetTopCardState through 01470890 and returns it, else returns 0. |

**GreaterEqualExpression** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0149a210` | GreaterEqualExpression::GreaterEqualExpression | confirmed | Constructor body: runs base constructor 0149adc0 and stores the GreaterEqualExpression vtable (018f962c). |
| `0149a290` | GreaterEqualExpression::GetTypeName | confirmed | Virtual slot [11] of GreaterEqualExpression. Returns the literal 'GreaterEqualExpression'. |
| `0149a320` | GreaterEqualExpression::AssertFalseStub | confirmed | Virtual slot [15] of GreaterEqualExpression. Calls the assert helper 012f5a60 with 'false', source GreaterEqualExpression.cpp and line 0x12, then returns 0. |
| `0149a340` | GreaterEqualExpression::Evaluate | confirmed | Virtual slot [14] of GreaterEqualExpression. Traces '(greater equal'. Evaluates lhs (+0x1c) and rhs (+0x20) through slot [14]; a failed child evaluation traces… |
| `0149a560` | GreaterEqualExpression::ScalarDeletingDtor | confirmed | Virtual slot [0]: stores the GreaterEqualExpression vtable, calls base destructor 0149ae80, frees the object with _free when bit 0 of the argument is set. Retu… |
| `0149a4e0` | GreaterEqualExpression::Create | likely | Factory: operator new 00d833a6 with size 0x24; on null it returns 0, otherwise it constructs with 0149a210 and returns the object. |
| `0149a540` | GreaterEqualExpression::RegisterFactory | likely | Registers factory 0149a4e0 under classID 0x20 through 013ce130 and 013ce230. |

**GreaterThanExpression** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01499b40` | GreaterThanExpression::GreaterThanExpression | confirmed | Constructor body: runs base constructor 0149adc0 and stores the GreaterThanExpression vtable (018f9524). |
| `01499bc0` | GreaterThanExpression::GetTypeName | confirmed | Virtual slot [11] of GreaterThanExpression. Returns the literal 'GreaterThanExpression'. |
| `01499c50` | GreaterThanExpression::Evaluate | confirmed | Virtual slot [14] of GreaterThanExpression. Traces '(greater than'. Evaluates lhs child (+0x1c) and rhs child (+0x20) through their slot [14]. A non-integer lh… |
| `01499eb0` | GreaterThanExpression::ScalarDeletingDtor | confirmed | Virtual slot [0]: stores the GreaterThanExpression vtable, calls base destructor 0149ae80, frees the object with _free when bit 0 of the argument is set. Retur… |
| `01499f10` | GreaterThanExpression::GetValidTargets | confirmed | Virtual slot [15] of GreaterThanExpression. Traces '(gvt:GreaterThanExpression'. Walks the input card set; for each card it sets it as current (013814d0), eval… |
| `01499e30` | GreaterThanExpression::Create | likely | Factory: operator new 00d833a6 with size 0x24; on null it returns 0, otherwise it constructs with 01499b40 and returns the object. |
| `01499e90` | GreaterThanExpression::RegisterFactory | likely | Registers factory 01499e30 under classID 0x1e through 013ce130 and 013ce230. |

**HandleResponseState** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01412d30` | HandleResponseState::OnButtonPressed | confirmed | Virtual slot [40] shared by HandleResponseState (018e5884 [40]) and CWHandleResponseState (018ec444 [40]). Logs 'Handle Response State:: button Pressed?!' to F… |
| `01412f70` | HandleResponseState::TypeName | confirmed | Virtual slot [11] of the HandleResponseState vtable (018e5884): returns the address of the literal 'HandleResponseState' (6-byte body). |
| `01412ff0` | HandleResponseState::ScalarDeletingDtor | confirmed | Vtable slot [0] of HandleResponseState: calls the destructor body 01412e60 on this, frees this through _free when bit 0 of the flag argument is set, and return… |
| `014136f0` | HandleResponseState::HandleResponseState | confirmed | Calls base constructor FUN_0147c980 on this, stores HandleResponseState::vftable, zeroes the member vectors and counters at +0xfc..+0x10c and +0x140..+0x148, a… |
| `01413830` | HandleResponseState::CtorWithArg | confirmed | Calls base constructor FUN_0147c9f0 with its one stack argument, stores HandleResponseState::vftable, zeroes the member vectors and counters as in HandleRespon… |
| `014143c0` | HandleResponseState::EnterState | confirmed | Virtual slot [14] shared by HandleResponseState (018e5884 [14]) and CWHandleResponseState (018ec444 [14]). Logs 'HandleResponsState::enterState' (sic) and read… |
| `01414960` | HandleResponseState::LeaveState | confirmed | Virtual slot 18 of HandleResponseState::vftable. Logs the IN/END HANDLERESPONSE::LEAVESTATE trace via FUN_0145c4d0 and FUN_013508b0. If FUN_013f13f0 returns no… |
| `01414fa0` | HandleResponseState::Serialize | confirmed | Virtual slot 10 of HandleResponseState::vftable. Gets a writer object uVar5 (vtable+4 on this when param_2 is 0, otherwise vtable+8 with param_2). Then it appe… |
| `01412f80` | HandleResponseState::GetStaticDword | likely | Virtual slot [12] of the HandleResponseState vtable (018e5884): returns the dword at global DAT_01cbe250. |
| `01412f90` | HandleResponseState::SetStaticDword | likely | Virtual slot [13] of the HandleResponseState vtable (018e5884): stores its argument into global DAT_01cbe250. |
| `014132c0` | HandleResponseState::ForwardIfPassMatch | likely | Virtual slot [41] shared by HandleResponseState (018e5884 [41]) and CWHandleResponseState (018ec444 [41]). When param_3 is 1 it builds a std::string 'pass', lo… |
| `01413980` | HandleResponseState::New | likely | Allocates 0x15c bytes through operator new (00d833a6); when the allocation is non-null it runs the constructor 014136f0 on the block and returns the object, ot… |
| `014139f0` | HandleResponseState::RegisterFactory | likely | Static initializer: registers the HandleResponseState factory 01413980 under classID 0xfd (253) by calling FUN_013ce130 and FUN_013ce230 with that key and fact… |
| `01413a10` | HandleResponseState::NextState | likely | Runs one step of the response command queue. If the pending id at this+0x134 is set it calls vtable slot 18 (+0x48) on FUN_01347e10's result and clears it. For… |

**HasInstalledActions** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014122e0` | HasInstalledActions::DtorBody | confirmed | Stores HasInstalledActions::vftable into this, then calls the tree teardown 01412270 on the member and returns. Called from four unwind funclets and from 0139b… |
| `014123a0` | HasInstalledActions::Ctor | confirmed | Stores HasInstalledActions::vftable into this and calls the tree member initialiser 01412330, then returns this. |
| `01412400` | HasInstalledActions::ScalarDeletingDtor | confirmed | Stores HasInstalledActions::vftable into this, calls the tree teardown 01412270, and frees this through _free when bit 0 of the flag argument is set; returns t… |
| `01411c70` | HasInstalledActions::EraseMatchingEntry | likely | Virtual slot [2] shared by CTP::Game (018dcda4 [2]), Card (018de528 +0x48 [2]), CWCard (018e5328 +0x48 [2]) and HasInstalledActions (018e57b4 [2]). Walks the t… |

**HasProperties** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01423fd0` | HasProperties::GuardedForwardSlot1 | confirmed | Virtual slot 1 of HasProperties::vftable. Asserts 'mPropertySet' non-null at HasProperties.cpp line 9 when this+4 is zero, then calls 013111d0 with its two sta… |
| `01424010` | HasProperties::GuardedForwardSlot2 | confirmed | Virtual slot 2 of HasProperties::vftable. Asserts 'mPropertySet' non-null at HasProperties.cpp line 0x11 when this+4 is zero, then calls 013111d0 with its two … |
| `01424050` | HasProperties::GuardedForwardSlot4 | confirmed | Virtual slot 4 of HasProperties::vftable. Asserts 'mPropertySet' non-null at HasProperties.cpp line 0x18 when this+4 is zero, then calls 01312380(arg1) and 013… |
| `014240c0` | HasProperties::GuardedForwardSlot5 | confirmed | Virtual slot 5 of HasProperties::vftable. Asserts 'mPropertySet' non-null at HasProperties.cpp line 0x25 when this+4 is zero, then calls 013110e0(arg1). |
| `014240f0` | HasProperties::Ctor | confirmed | Constructor: stores HasProperties::vftable at +0, allocates a 0x10-byte block through 00d833a6 and, when that is non-null, calls 01312930 and stores its result… |
| `01424160` | HasProperties::DtorBody | confirmed | Destructor body: resets the vtable to HasProperties::vftable, and when +4 (mPropertySet) is non-null calls its vtable slot 0 with 1 and clears +4. |
| `014241e0` | HasProperties::ScalarDeletingDtor | confirmed | Virtual slot 0 of HasProperties::vftable with the scalar deleting destructor shape: resets the vtable, calls vtable slot 0 of +4 with 1 and clears +4, then cal… |
| `01424190` | HasProperties::GetPropertySet | likely | Virtual slot 7 of HasProperties::vftable: returns the dword at this+4 (mov eax,[ecx+4]; ret). |
| `014241a0` | HasProperties::PropertySetJumpSlot8 | likely | Virtual slot 8 of HasProperties::vftable: loads this+4 into ECX (mov ecx,[ecx+4]) and jumps to 01312780 (e9 ...). |
| `014241b0` | HasProperties::PropertySetJumpSlot9 | likely | Virtual slot 9 of HasProperties::vftable: loads this+4 into ECX (mov ecx,[ecx+4]) and jumps to 01312810 (e9 ...). |

**HasPropertyExpression** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01499780` | HasPropertyExpression::HasPropertyExpression | confirmed | Constructor body: runs base constructor 0149adc0, stores the HasPropertyExpression vtable (018f945c), and zeroes +0x24 (property id) and +0x28 (target index). |
| `01499800` | HasPropertyExpression::GetTypeName | confirmed | Virtual slot [11] of HasPropertyExpression. Returns the literal 'HasPropertyExpression'. |
| `01499890` | HasPropertyExpression::AssertFalseStub | confirmed | Virtual slot [15] of HasPropertyExpression. Calls the assert helper 012f5a60 with 'false', source HasPropertyExpression.cpp and line 0x17, then returns 0. |
| `014998b0` | HasPropertyExpression::Evaluate | confirmed | Virtual slot [14] of HasPropertyExpression. Traces '(hasProperty %d' with the property id. Gets the environment (01381380), calls its virtual +0x200 to resolve… |
| `01499ae0` | HasPropertyExpression::ScalarDeletingDtor | confirmed | Virtual slot [0]: stores the HasPropertyExpression vtable, calls base destructor 0149ae80, frees the object with _free when bit 0 of the argument is set. Retur… |
| `01499a60` | HasPropertyExpression::Create | likely | Factory: operator new 00d833a6 with size 0x2c; on null it returns 0, otherwise it constructs with 01499780 and returns the object. |
| `01499ac0` | HasPropertyExpression::RegisterFactory | likely | Registers factory 01499a60 under classID 0x1b1 through 013ce130 and 013ce230. |

**IfThenExpression** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01499320` | IfThenExpression::IfThenExpression | confirmed | Constructor body: runs base constructor 0149adc0, stores the IfThenExpression vtable (018f9354), and zeroes the dword at +0x24 (the optional else child). |
| `014993a0` | IfThenExpression::GetTypeName | confirmed | Virtual slot [11] of IfThenExpression. Returns the literal 'IfThenExpression'. |
| `01499550` | IfThenExpression::Evaluate | confirmed | Virtual slot [14] of IfThenExpression. Evaluates the condition child (+0x1c) through slot [14]. In the ALL state (013815e0 returns 3) it traces 'In an ALL eval… |
| `01499710` | IfThenExpression::ScalarDeletingDtor | confirmed | Virtual slot [0]: calls the destructor body 014993e0, then frees the object with _free when bit 0 of the argument is set. Returns this. |
| `014993e0` | IfThenExpression::~IfThenExpression | likely | Destructor body: stores the IfThenExpression vtable; if the child at +0x24 is non-null it calls that child's virtual slot 0 with flag 1 (deleting destructor) a… |
| `01499460` | IfThenExpression::GetValidTargets | likely | Virtual slot [15] of IfThenExpression. Traces '(gvt:IfThen'. Calls the condition child at +0x1c through slot [14]. If it is true, traces and calls the then-chi… |
| `01499690` | IfThenExpression::Create | likely | Factory: calls operator new 00d833a6 with size 0x28; on null it returns 0, otherwise it constructs with 01499320 and returns the object. |
| `014996f0` | IfThenExpression::RegisterFactory | likely | Registers factory 01499690 under classID 0x1c through 013ce130(0x1c, factory) and 013ce230(0x1c, factory). |

**IntroduceCardState** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `015151d0` | IntroduceCardState::StateName | confirmed | Virtual slot 11 of IntroduceCardState. Returns the address of the literal string IntroduceCardState (mov eax, imm32; ret). |
| `01515380` | IntroduceCardState::SynchPointReached | confirmed | Virtual slot 44 of IntroduceCardState. Formats the IntroduceCardState::synchPointReached trace text through FUN_0145c4d0, passes it to FUN_013508b0 and returns… |
| `015153a0` | IntroduceCardState::RestoreState | confirmed | Virtual slot 17 of IntroduceCardState. Formats the IntroduceCardState::restoreState trace text, passes it to FUN_013508b0 and returns 2. |
| `015153c0` | IntroduceCardState::LeaveState | confirmed | Virtual slot 18 of IntroduceCardState. Formats the IntroduceCardState::leaveState trace text, passes it to FUN_013508b0, calls FUN_013f39c0, and returns 1. |
| `01515480` | IntroduceCardState::ScalarDeletingDtor | confirmed | Virtual slot 0 of IntroduceCardState. Stores the vtable, calls FUN_0147c520 (base destructor), then calls _free on this when bit 0 of the second argument is se… |
| `015154e0` | IntroduceCardState::ConstructWithNameString | confirmed | Calls FUN_0147c980 on this, stores IntroduceCardState::vftable into *this, then assigns the 18-byte literal IntroduceCardState into the std::string member at t… |
| `015153f0` | IntroduceCardState::ReadState | likely | Virtual slot 9 of IntroduceCardState. Calls vtbl+0x10 with the stream and an output pointer; if that succeeds it calls FUN_0147c850 on the argument and then re… |
| `01515430` | IntroduceCardState::WriteState | likely | Virtual slot 10 of IntroduceCardState. Selects the serializer like WriteState of ProcessEventEmissionStateForActivePlayer (vtbl+4 when the argument is 0, else … |
| `01515550` | IntroduceCardState::ConstructWithArgAndName | likely | Calls FUN_0147c9f0 with the argument, stores IntroduceCardState::vftable into *this, assigns the literal IntroduceCardState into the std::string at this+0x1c a… |
| `015155c0` | IntroduceCardState::Create | likely | Allocates 0xf8 bytes with FUN_00d833a6; when the allocation is nonnull it constructs the object with FUN_015154e0 and returns it, otherwise returns 0. |

**IsBorneByExpression** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01498e10` | IsBorneByExpression::IsBorneByExpression | confirmed | Constructor body: runs the base constructor 0149adc0 under an EH frame, then stores the IsBorneByExpression vtable (018f92a4) into the object's first dword. |
| `01498e90` | IsBorneByExpression::GetTypeName | confirmed | Virtual slot [11] of IsBorneByExpression. Returns the literal string 'IsBorneByExpression'. |
| `01498f20` | IsBorneByExpression::AssertFalseStub | confirmed | Virtual slot [15] of IsBorneByExpression. Calls the assert helper 012f5a60 with 'false', the source file IsBorneByExpression.cpp and line 0x14, then returns 0. |
| `01498fc0` | IsBorneByExpression::ScalarDeletingDtor | confirmed | Virtual slot [0]: stores the IsBorneByExpression vtable, calls base destructor 0149ae80, then frees the object with _free when bit 0 of the argument is set. Re… |
| `01499020` | IsBorneByExpression::Evaluate | confirmed | Virtual slot [14] of IsBorneByExpression. Traces '(IsBorneBy'. Calls child +0x1c through its slot 15 to fill a card list; when that returns 0 it traces ') => f… |
| `01498f40` | IsBorneByExpression::Create | likely | Factory: calls operator new 00d833a6 with size 0x24; if the allocation is null it returns 0, otherwise it constructs the node with 01498e10 and returns it. |
| `01498fa0` | IsBorneByExpression::RegisterFactory | likely | Registers factory 01498f40 under classID 0x87 by calling 013ce130(0x87, factory) and 013ce230(0x87, factory). |

**LastActionQueryNode** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01498800` | LastActionQueryNode::GetValidTargets | likely | Virtual slot [15] of LastActionQueryNode. Traces '(gvt:LastActionQueryNode', switches on the event kind at this+0x1c and picks the event card with a per-kind h… |
| `01498bf0` | LastActionQueryNode::MatchEventCard | likely | Virtual slot [20] of LastActionQueryNode, called by slot 15 with the event card id. Traces 'Event Card is null!' and returns 0 for a null card. Otherwise it wa… |

**MacroNode** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01423400` | MacroNode::Ctor | confirmed | Constructor: calls base init 0149ab30 on this (ECX), stores MacroNode::vftable at +0, zeroes the dword at +0x10, and returns this. |
| `01423480` | MacroNode::GetClassName | confirmed | Virtual slot 11 of MacroNode::vftable; returns the address of the literal 'MacroNode' (bytes b8 10 71 8e 01, c3). |
| `014234c0` | MacroNode::DtorBody | confirmed | Destructor body: stores MacroNode::vftable at +0, zeroes the dword at +0x10, and calls 0149a860 on this (base teardown, unlabelled). |
| `01423540` | MacroNode::GetTarget | confirmed | Virtual slot 20 of MacroNode::vftable (also slot 20 of CWMacroNode::vftable). Traces 'MacroNode getTarget %d' to param_2 via 013846b0, then switches on the tar… |
| `014237b0` | MacroNode::ReturnTrueTraced | confirmed | Virtual slot 15 of MacroNode::vftable: passes '(gvt:MacroNode' to 01385610 and ') => true' to 01384ee0 on param_2, then returns 1 (mov al,1; ret 8). |
| `01423840` | MacroNode::WriteToSerial | confirmed | Virtual slot 10 of MacroNode::vftable: calls vtable +8 with param_2, passes the 'Starting MacroNode', 'Macro: ' and 'Ending MacroNode' texts to the no-op 013c2… |
| `01423a50` | MacroNode::ScalarDeletingDtor | confirmed | Virtual slot 0 of MacroNode::vftable: calls dtor body 014234c0 on this, then calls _free(this) when bit 0 of the flag argument is set, and returns this. |
| `01423c30` | MacroNode::MatchQueryIter | confirmed | Virtual slot 14 of MacroNode::vftable. Writes '(MacroNode' via 01385610, switches on the macro kind at this+0x10: 1 'This Macro' (origin card id from 01340370 … |
| `014237e0` | MacroNode::ReadFromSerial | likely | Virtual slot 9 of MacroNode::vftable. Calls vtable +0x10 with (param_2, &local), calls 0149acf0(param_2) as a check, reads an integer via 013c24d0 into local, … |
| `014239d0` | MacroNode::Create | likely | Calls 00d833a6 with size 0x14 and, when the result is non-null, constructs a MacroNode in it with 01423400; returns that pointer or 0. |

**MultiActionState** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0146e940` | MultiActionState::enterState | confirmed | Calls vtable +0x4c with no argument, logs 'MultiActionState::enterState', and reads the engine return map (013f37e0): integer keys kPlayerReturn (4) into this+… |

**PhaseState** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01468d70` | PhaseState::~PhaseState | confirmed | Destructor body (fastcall, ECX = this). Stores PhaseState::vftable into the object first, then calls 0145d080 (base-class teardown) and returns. Frees nothing. |
| `01468de0` | PhaseState::ClassName | confirmed | Virtual slot [11] (+0x2c) of PhaseState. Returns the address of the literal 'PhaseState' (bytes: returns the string pointer, 6 bytes). |
| `01468f20` | PhaseState::~PhaseState | confirmed | Virtual slot [0] of PhaseState: scalar deleting destructor (one stack flag). Stores PhaseState::vftable, calls 0145d080 (base-class teardown), then frees the o… |
| `01468fd0` | PhaseState::updateState | confirmed | Virtual slot [15] (+0x3c) of PhaseState (fastcall, ECX = this). Traces '%s::updateState: %d' with the name and the state at +0xf4 through 013508b0. Switch on t… |
| `01468e30` | PhaseState::LoadFromStream | likely | Virtual slot [9] (+0x24) of PhaseState (thiscall, one stack argument, the stream, no return value). Calls slot [4] (+0x10) of this with the stream and a local;… |
| `01468ec0` | PhaseState::SaveToStream | likely | Virtual slot [10] (+0x28) of PhaseState (thiscall, one stack argument, the output object). Calls slot [2] (+0x8) of this with the argument, traces 'Starting Ph… |
| `01468f80` | PhaseState::enterState | likely | Virtual slot [14] (+0x38) of PhaseState (fastcall, ECX = this). Calls slot [19] (+0x4c) of this, traces '%s::enterState' with the object's name string (at +0x2… |

**PickListState** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0142c570` | PickListState::PickListState | confirmed | Default constructor: calls FUN_0147c980 (base constructor), stores PickListState::vftable at offset 0, zeroes the dwords at +0x10c, +0x110 and +0x114, sets the… |
| `0142c680` | PickListState::GetName | confirmed | Returns the address of the literal 'PickListState' (6 bytes, mov eax, 018e7de4; ret). |
| `0142c690` | PickListState::GetSharedDword | confirmed | Returns the dword global at 01cbe290 (bytes a1 90 e2 cb 01 c3). |
| `0142c6a0` | PickListState::SetSharedDword | confirmed | Stores its stack argument into the dword global at 01cbe290 (bytes 8b 44 24 04 / a3 90 e2 cb 01 / c2 04 00). |
| `0142c7b0` | PickListState::Dtor | confirmed | Stores PickListState::vftable, calls FUN_01300cd0, frees the string at +0x120 when its capacity at +0x134 is above 15 and resets it; when the array pointer at … |
| `0142dd30` | PickListState::ScalarDeletingDtor | confirmed | Calls FUN_0142c7b0 (the destructor body), then frees the object with _free when bit 0 of the second argument is set; returns the object pointer. |
| `0142c6c0` | PickListState::PickListStateWithArg | likely | Argument constructor: calls FUN_0147c9f0 with its argument (base constructor), then the same field setup as 0142c570 (vtable, zeroed bounds and flags, capacity… |
| `0142c890` | PickListState::ReceiveList | likely | Takes the pick-list string range [param_3, param_4) and a list object; for each entry it either parses an integer with _atol (when the dword at param_1+0x118 i… |
| `0142dca0` | PickListState::Create | likely | Allocates 0x14c bytes with FUN_00d833a6; when the allocation succeeds it runs FUN_0142c570 (PickListState default constructor) on the block and returns its res… |

**PilePlayArea** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0139d3d0` | PilePlayArea::Ctor | confirmed | Thiscall constructor: calls the PlayArea constructor 0139d2a0, stores the PilePlayArea primary vtable 018de92c at this and the secondary 018de8ec at this+8, se… |
| `0139d4a0` | PilePlayArea::DtorBody | confirmed | Thiscall destructor body: stores the PilePlayArea primary vtable 018de92c and the secondary 018de8ec, then calls the PlayArea destructor body 0139d370. |
| `0139e190` | PilePlayArea::ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor: calls PilePlayArea::DtorBody 0139d4a0 on this, then frees the object through the _free thunk when the flag bit 0 is set; r… |
| `0139e0f0` | PilePlayArea::Factory | likely | Allocates 0x48 bytes through 00d833a6; when non-null constructs the PilePlayArea through 0139d3d0 and returns the result; returns 0 when allocation fails. |

**PlayArea** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0139d2a0` | PlayArea::Ctor | confirmed | Thiscall constructor: calls the base constructor 01343d20, stores the PlayArea primary vtable 018de894 at this and the secondary vtable 018de858 at this+8, set… |
| `0139d370` | PlayArea::DtorBody | confirmed | Thiscall destructor body: stores the PlayArea primary vtable 018de894 at this and the secondary 018de858 at this+8, then calls the base destructor body 01344d1… |
| `0139e0d0` | PlayArea::ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor: calls PlayArea::DtorBody 0139d370 on this, then when bit 0 of the flag argument is set calls _free(this); returns this wit… |
| `0139e030` | PlayArea::Factory | likely | Allocates 0x48 bytes through 00d833a6; when non-null constructs the PlayArea through 0139d2a0 and returns the object pointer + 8; returns 0 when allocation fai… |
| `0139e560` | PlayArea::ForEachElementSlot13 | likely | Virtual slot 13 of PlayArea (offset 0x34; also PilePlayArea slot 13). Walks the element list headed at this+0x10 and calls each element's vslot +0x34 (slot 13)… |
| `0139e760` | PlayArea::CountCards | likely | Virtual slot 17 of PlayArea (shared with PilePlayArea, PlayerPlayArea and CWPlayerPlayArea). Builds a temporary list through the object's slot 13 (+0x34), emit… |

**PlayCardState** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01467560` | PlayCardState::~PlayCardState | confirmed | Destructor body (thiscall). Stores PlayCardState::vftable into the object, then calls the base destructor 0147c520. Frees nothing and does not free the object. |
| `014675d0` | PlayCardState::ClassName | confirmed | Virtual slot [11] (+0x2c) of PlayCardState. Returns the address of the literal 'PlayCardState' (bytes: mov eax, string; ret). |
| `01467620` | PlayCardState::updateHints | confirmed | Virtual slot [68] (+0x110) of PlayCardState (one stack argument, ret 4, argument unused). Traces 'PlayCardState::updateHints' through 0145c4d0 and 013508b0 and… |
| `01467640` | PlayCardState::ActionPlayed | confirmed | Virtual slot [25] (+0x64) of PlayCardState (thiscall, three stack arguments). Traces the call, then checks that 01344e20() equals param_2, else traces 'Got an … |
| `01467b20` | PlayCardState::~PlayCardState | confirmed | Virtual slot [0] of PlayCardState: scalar deleting destructor (one stack flag). Stores PlayCardState::vftable, calls the base destructor 0147c520, then frees t… |
| `01467b80` | PlayCardState::Ctor | confirmed | Default constructor (fastcall, ECX = this). Runs base constructor 0147c980, stores PlayCardState::vftable, assigns the name 'PlayCardState' (0xd chars) into th… |
| `01468060` | PlayCardState::buttonPressedWithString | confirmed | Virtual slot [41] (+0xa4) of PlayCardState (thiscall, three stack arguments; the text is the fourth). Traces the call with the player id, the button id and the… |
| `01468600` | PlayCardState::CardPlayed | confirmed | Virtual slot [21] (+0x54) of PlayCardState (thiscall, two stack arguments). Checks 01344e20() equals param_2; otherwise traces 'Got an cardPlayed from %d inste… |
| `01468a20` | PlayCardState::BeginCardPlayed | confirmed | Virtual slot [23] (+0x5c) of PlayCardState (thiscall, three stack arguments; the last is passed on). Checks the player (01344e20 against param_2; mismatch trac… |
| `01467970` | PlayCardState::BailIfGameEnded | likely | Virtual slot [17] (+0x44) of PlayCardState (fastcall, ECX = this). Calls 0145c550 and 013f7e80; when that predicate is true it calls 0145c550(0) and 013f7e90(0… |
| `01467a00` | PlayCardState::LoadFromStream | likely | Virtual slot [9] (+0x24) of PlayCardState (thiscall, one stack argument, the stream). Calls slot [4] (+0x10) with the stream and a local; on success calls base… |
| `01467c00` | PlayCardState::CtorWithBaseArg | likely | Constructor overload (thiscall, one stack argument, ret 4). Passes its argument to base constructor 0147c9f0, then stores PlayCardState::vftable, assigns the n… |

**PlayElement** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01342250` | PlayElement::RemoveSuppressEffect | confirmed | Thiscall with one ValueData pointer argument (ret 4). When its type word is 2 (integer) it replaces the stack argument with the payload at +8 and tail-jumps to… |
| `01341ec0` | PlayElement::RemoveIntegerListEntry | likely | Thiscall (ECX = this) with one stack argument (ret 4) that is the int key. It reads the integer-list property 0xa2 through vtable slot +4, asserting keywordDat… |
| `013432f0` | PlayElement::CollectCardsFromNodes | likely | Walks the tree whose head is at this+0x10 (increment 01340aa0). For each node it visits every element pointer of the node's vector (begin +0x14, end +0x18), dy… |

**PlaySingleCardState** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01465fb0` | PlaySingleCardState::ValidateButtonPressed | confirmed | Virtual slot [40] (+0xa0) of PlaySingleCardState (thiscall, two stack arguments). Compares param_2 with the id returned by 01344e20; on a mismatch it traces 'G… |
| `01466050` | PlaySingleCardState::~PlaySingleCardState | confirmed | Destructor body (thiscall). Stores PlaySingleCardState::vftable into the object, frees the buffer at this+0xfc when non-null, zeroes the pointer and the two fo… |
| `014660f0` | PlaySingleCardState::ClassName | confirmed | Virtual slot [11] (+0x2c) of PlaySingleCardState. Returns the address of the literal 'PlaySingleCardState' (bytes: mov eax, 018f0440; ret). |
| `01466130` | PlaySingleCardState::ActionPlayed | confirmed | Virtual slot [25] (+0x64) of PlaySingleCardState (thiscall, three stack arguments). Traces the call, then searches the 4-byte id vector at +0xfc..+0x100 for pa… |
| `01466430` | PlaySingleCardState::~PlaySingleCardState | confirmed | Virtual slot [0] of PlaySingleCardState: scalar deleting destructor (ret 4). Calls the destructor body 01466050, then if bit 0 of its flag argument is set free… |
| `01466450` | PlaySingleCardState::buttonPressedWithString | confirmed | Virtual slot [41] (+0xa4) of PlaySingleCardState (thiscall, three stack arguments). Traces the call with the player id, the button id and the text. If 01344e20… |
| `01466730` | PlaySingleCardState::Ctor | confirmed | Default constructor (fastcall, ECX = this). Runs base constructor 0147c980, stores PlaySingleCardState::vftable, zeroes the dwords at +0xfc, +0x100 and +0x104,… |
| `01466840` | PlaySingleCardState::CardPlayed | confirmed | Virtual slot [21] (+0x54) of PlaySingleCardState (thiscall, two stack arguments). Checks that 01344e20() equals param_2, else traces 'Got an cardPlayed from %d… |
| `01466d00` | PlaySingleCardState::BeginCardPlayed | confirmed | Virtual slot [23] (+0x5c) of PlaySingleCardState (thiscall, three stack arguments; the last is passed on). Same player and target checks as CardPlayed (0146684… |
| `01467140` | PlaySingleCardState::updateHints | confirmed | Virtual slot [68] (+0x110) of PlaySingleCardState (fastcall, ECX = this, no return value). Walks the list returned by 01347e30 (begin at +4, end at +8). For ea… |
| `014667b0` | PlaySingleCardState::CtorWithBaseArg | likely | Constructor overload (thiscall, one stack argument, ret 4). Passes its argument to base constructor 0147c9f0, then does the same field setup as PlaySingleCardS… |
| `01467500` | PlaySingleCardState::LoadFromStream | likely | Virtual slot [9] (+0x24) of PlaySingleCardState (thiscall, one stack argument, the stream). Calls slot [4] (+0x10) of this with the stream; if that succeeds ca… |

**PlayerPlayArea** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0139d4f0` | PlayerPlayArea::Ctor | confirmed | Thiscall constructor: calls the PlayArea constructor 0139d2a0, stores the PlayerPlayArea primary vtable 018de9c4 at this and the secondary 018de988 at this+8, … |
| `0139e250` | PlayerPlayArea::ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor: calls PlayerPlayArea::DtorBody 0139d5c0 on this, then frees the object through the _free thunk when the flag bit 0 is set;… |
| `0139d5c0` | PlayerPlayArea::DtorBody | likely | Thiscall destructor body: stores the PlayerPlayArea vtables 018de9c4 and 018de988, then for each of the three child pointers at +0x48, +0x4c and +0x50 that is … |
| `0139d680` | PlayerPlayArea::CreateChildPlayAreas | likely | Virtual slot 16 of PlayerPlayArea (offset 0x40). Calls 01340230 on the argument, then creates three PilePlayArea children: each is operator new(0x48) through 0… |
| `0139db10` | PlayerPlayArea::ForwardSlot13ToChildren | likely | Virtual slot 13 of PlayerPlayArea (offset 0x34). Calls the child at this+0x48 vslot +0x34 (slot 13) with the argument, then the same for the children at +0x4c … |
| `0139e1b0` | PlayerPlayArea::Factory | likely | Allocates 0x54 bytes through 00d833a6; when non-null constructs the PlayerPlayArea through 0139d4f0 and returns the result; returns 0 when allocation fails. |

**ProcessEventEmissionState** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014643b0` | ProcessEventEmissionState::enterState | confirmed | Virtual slot [14] of ProcessEventEmissionState (also slot [14] of CWProcessEventEmissionState). Traces the method name, calls its own virtual slot [19] (+0x4c)… |
| `01464440` | ProcessEventEmissionState::leaveState | confirmed | Virtual slot [18] of ProcessEventEmissionState. Traces the method name, calls 0145c550(0) and 013fc390, then 01347a70 and 013f39c0. It passes the value stored … |
| `01464a50` | ProcessEventEmissionState::~ProcessEventEmissionState | confirmed | Destructor body (thiscall). Stores ProcessEventEmissionState::vftable into the object, frees the buffer at this+0x118 and the buffer at this+0x108 when they ar… |
| `01464b20` | ProcessEventEmissionState::ClassName | confirmed | Virtual slot [11] of ProcessEventEmissionState. Returns the address of the literal string 'ProcessEventEmissionState' (6 bytes of code, no arguments). |
| `01464b60` | ProcessEventEmissionState::~ProcessEventEmissionState | confirmed | Virtual slot [0] of ProcessEventEmissionState: scalar deleting destructor. Calls the destructor body 01464a50, then if bit 0 of param_2 is set frees the object… |
| `01464d40` | ProcessEventEmissionState::Ctor | confirmed | Default constructor (thiscall, no stack arguments). Runs base constructor 0147c980, stores ProcessEventEmissionState::vftable, zeroes the dword fields at +0xf8… |
| `01464fb0` | ProcessEventEmissionState::nextState | confirmed | Virtual slot [73] (+0x124) of ProcessEventEmissionState; the target that enterState tail-calls. On the first call (byte +0x130 zero) it traces 'nextState (init… |
| `0150ffe0` | ProcessEventEmissionState::testActionPreconditions | confirmed | Traces 'ProcessEventEmissionState::testActionPreconditions(%d, %d - %d)' through 013508b0 (a logger that emits text only when its flags are set), calls the arg… |
| `01464640` | ProcessEventEmissionState::testActionPreconditions | likely | Thiscall(param_2 object, param_3). Traces the method through 0145c4d0 and 013508b0, calls virtual slot [10] (+0x28) of param_2 and sets up a 136-byte local buf… |
| `01464e30` | ProcessEventEmissionState::CtorWithBaseArg | likely | Constructor overload (thiscall, one stack argument, ret 4). Passes its argument to base constructor 0147c9f0, then does the same field initialization as Proces… |
| `01465430` | ProcessEventEmissionState::LoadFromStream | likely | Virtual slot [9] (+0x24) of ProcessEventEmissionState (thiscall, one stack argument, the stream). Calls slot [4] (+0x10) of this first and returns 0 if it fail… |
| `0150fd50` | ProcessEventEmissionState::enterState | likely | Slot [14] of ProcessEventEmissionStateForActivePlayer::vftable (01910a8c) and CWProcessEventEmissionStateForActivePlayer (01911284). Traces 'ProcessEventEmissi… |

**ProcessEventEmissionStateForActivePlayer** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `015104d0` | ProcessEventEmissionStateForActivePlayer::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape: calls the destructor body 015102e0, tests bit 0 of the stack argument and, when set, frees this with _free (00d82463); return… |
| `015106b0` | ProcessEventEmissionStateForActivePlayer::ProcessEventEmiss… | confirmed | Default constructor (fastcall): calls base ctor 0147c980, stores the vftable 01910a8c at offset 0, zeroes dwords +0x108, +0x10c, +0x110, +0x118, +0x11c, +0x120… |
| `015107a0` | ProcessEventEmissionStateForActivePlayer::ProcessEventEmiss… | confirmed | Constructor overload with one stack argument: calls base ctor 0147c9f0 with it, stores the vftable 01910a8c, zeroes the same field set as the default construct… |
| `0150fe80` | ProcessEventEmissionStateForActivePlayer::CheckOrDelegatePr… | likely | Slot [30] of ProcessEventEmissionStateForActivePlayer::vftable (01910a8c). Returns 1 when the argument equals the dword at this+0x140, or when this+0x144 is no… |
| `015102e0` | ProcessEventEmissionStateForActivePlayer::~ProcessEventEmis… | likely | Stores the vftable 01910a8c at offset 0, frees the buffer at +0x118 when non-null and zeroes +0x118..+0x120, frees the buffer at +0x108 when non-null and zeroe… |
| `01510da0` | ProcessEventEmissionStateForActivePlayer::ReadState | likely | Virtual slot 9. Calls vtbl+0x10 with the stream and an output pointer and returns 0 if that fails. Then reads integers via FUN_013c24d0 into this+0xf8, +0xfc, … |

**QueryPlayabilityNode** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01492a00` | QueryPlayabilityNode::TypeName | confirmed | No arguments. Returns the literal "QueryPlayabilityNode" (mov eax, 018f7e3c; ret). |
| `01492d80` | QueryPlayabilityNode::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape (ret 4): calls the body 01492a40 (ECX=this), then when bit 0 of the flag argument is set calls _free (00d82463 thunk) on this. |
| `01492980` | QueryPlayabilityNode::Ctor | likely | Thiscall. Calls the base constructor 01494290 (ECX=this), installs the QueryPlayabilityNode vtable (018f7dec) and sets this+0x1c, +0x20, +0x24 and +0x28 to 0. |
| `01492a10` | QueryPlayabilityNode::GetStaticDword | likely | No arguments. Returns the dword global DAT_01cbe5f0. |
| `01492a20` | QueryPlayabilityNode::SetStaticDword | likely | One stack argument, ret 4. Stores the argument into the dword global DAT_01cbe5f0. |
| `01492a40` | QueryPlayabilityNode::DtorBody | likely | Non-freeing destructor body: installs the QueryPlayabilityNode vtable, calls vtable slot 0 with flag 1 on the objects at +0x24 and +0x28 when they are non-null… |
| `01492d00` | QueryPlayabilityNode::Factory | likely | Calls operator new with 0x2c bytes (00d833a6); when non-null calls the constructor 01492980 on it and returns the object, otherwise returns null. |
| `01492d60` | QueryPlayabilityNode::RegisterFactory | likely | Pushes the factory 01492d00 and the classID 0x35 (53), calls 013ce130, then calls 013ce230 with its result in ECX. Returns. |

**QueryPropertiesNode** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01491f50` | QueryPropertiesNode::TypeName | confirmed | No arguments. Returns the literal "QueryPropertiesNode" (mov eax, 018f797c; ret). |
| `01492220` | QueryPropertiesNode::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape (ret 4): calls the body 01491f90 (ECX=this), then when bit 0 of the flag argument is set calls _free (00d82463 thunk) on this. |
| `01491eb0` | QueryPropertiesNode::Ctor | likely | Thiscall. Calls the base QueryNode constructor 01494290 (ECX=this), constructs the embedded value member at +0x24 with 012fa910, installs the QueryPropertiesNo… |
| `01491f60` | QueryPropertiesNode::GetStaticDword | likely | No arguments. Returns the dword global DAT_01cbe5ec. |
| `01491f70` | QueryPropertiesNode::SetStaticDword | likely | One stack argument, ret 4. Stores the argument into the dword global DAT_01cbe5ec. |
| `01491f90` | QueryPropertiesNode::DtorBody | likely | Non-freeing destructor body: installs the QueryPropertiesNode vtable, destroys the embedded value at +0x24 through 01300cd0 (ValueData destructor body), then c… |
| `01492050` | QueryPropertiesNode::TraceAndReturnTrue | likely | Virtual slot 15, thiscall, two stack args (ignored, env). Traces '(gvt:QueryPropertiesNode' on the env argument, then ') => true', and returns 1. No other effe… |
| `014921a0` | QueryPropertiesNode::Factory | likely | Calls operator new with 0x30 bytes (00d833a6); when non-null calls the constructor 01491eb0 on it and returns the object, otherwise returns null. |
| `01492200` | QueryPropertiesNode::RegisterFactory | likely | Pushes the factory 014921a0 and the classID 0x2b (43), calls 013ce130, then calls 013ce230 with its result in ECX. Returns. |

**RepeatExpression** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01491ae0` | RepeatExpression::TypeName | confirmed | No arguments. Returns the literal "RepeatExpression" (mov eax, 018f7894; ret). |
| `01491ba0` | RepeatExpression::Evaluate | confirmed | Virtual slot 14, thiscall, two stack args (out ValueData, env). Traces '(Repeat', evaluates the count child at this+0x1c through its slot 14, takes its integer… |
| `01491e50` | RepeatExpression::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape (ret 4): installs the RepeatExpression vtable, calls the base destructor body 0149ae80 (ECX=this), then when bit 0 of the flag… |
| `01491a60` | RepeatExpression::Ctor | likely | Thiscall. Calls the base constructor 0149adc0 (ECX=this), installs the RepeatExpression vtable (018f7844) and returns this. Sets no fields of its own. |
| `01491af0` | RepeatExpression::GetStaticDword | likely | No arguments. Returns the dword global DAT_01cbe5e8. |
| `01491b00` | RepeatExpression::SetStaticDword | likely | One stack argument, ret 4. Stores the argument into the dword global DAT_01cbe5e8. |
| `01491b20` | RepeatExpression::DtorBody | likely | Non-freeing destructor body: installs the RepeatExpression vtable (018f7844), calls the base destructor body 0149ae80 (ECX=this) and returns. It does not call … |
| `01491b70` | RepeatExpression::TraceAndReturnFalse | likely | Virtual slot 15, thiscall, two stack args (ignored, env). Traces '(gvt: repeat' on the env argument, then ') => false', and returns 0. No other effect. |
| `01491dd0` | RepeatExpression::Factory | likely | Calls operator new with 0x24 bytes (00d833a6); when non-null calls the constructor 01491a60 on it and returns the object, otherwise returns null. |
| `01491e30` | RepeatExpression::RegisterFactory | likely | Pushes the factory 01491dd0 and the classID 0x16f (367), calls 013ce130, then calls 013ce230 with its result in ECX. Returns. |

**ReshuffleState** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01463860` | ReshuffleState::EnterState | confirmed | Virtual slot 14. Traces ReshuffleState::enterState through 0145c4d0 and 013508b0, calls vtable +0x4c (slot 19), asserts that returnValueMap exists, reads integ… |
| `01463900` | ReshuffleState::RestoreState | confirmed | Virtual slot 17. Traces ReshuffleState::restoreState through 0145c4d0 and 013508b0 and returns 2. |
| `01463a00` | ReshuffleState::LeaveState | confirmed | Virtual slot 18. Traces ReshuffleState::leaveState through 0145c4d0 and 013508b0, calls 013f39c0 with ECX = this+8 (the machine), and returns 1. |
| `01463af0` | ReshuffleState::~ReshuffleState | confirmed | Destructor body. Stores ReshuffleState::vftable, frees the string buffer at this+0xfc when its capacity (+0x110) is above 15, resets the string fields, then ca… |
| `01463ba0` | ReshuffleState::GetTypeName | confirmed | Virtual slot 11. Returns a pointer to the literal ReshuffleState. |
| `01463bb0` | ReshuffleState::GetClassDword | confirmed | Virtual slot 12. Returns the dword in the global DAT_01cbe56c. |
| `01463bc0` | ReshuffleState::SetClassDword | confirmed | Virtual slot 13. Stores its stack argument into DAT_01cbe56c and returns with ret 4. |
| `01463be0` | ReshuffleState::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Calls the destructor body 01463af0 on this; when bit 0 of the argument is set it frees this through the free thunk … |
| `01463c00` | ReshuffleState::Ctor | confirmed | Constructor. Stores ReshuffleState::vftable, sets the string member at this+0xfc empty with capacity 0xf, and assigns the 14-byte name ReshuffleState into this… |
| `01463c80` | ReshuffleState::CtorWithArg | confirmed | Constructor overload that passes its argument to the base constructor 0147c9f0, then applies the same member initialisation and name assignment as ReshuffleSta… |

**ReturnValueMap** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01415920` | ReturnValueMap::ReconstructComponentFromDataInvalid | confirmed | Virtual slot 9 of ReturnValueMap::vftable. Passes the fixed text 'Invalid use of ReturnValueMap::reconstructComponentFromData!' to FUN_012f5dc0, then returns 0… |
| `01416bf0` | ReturnValueMap::Serialize | confirmed | Virtual slot 10 of ReturnValueMap::vftable. Writes the label Starting ReturnValueMap through FUN_013c26e0 (a no-op, so the label is not emitted), calls vtable+… |
| `014179e0` | ReturnValueMap::~ReturnValueMap | confirmed | Fastcall-style with ECX = this, plain ret. Stores ReturnValueMap::vftable into this, calls FUN_01417960 (DtorRecordListFree) with ECX = this+4 to free the reco… |
| `01417a70` | ReturnValueMap::GetTypeName | confirmed | Virtual slot 11 of ReturnValueMap::vftable. Returns the address of the text ReturnValueMap (bytes b8 imm32 c3, mov eax then ret). |
| `01417cd0` | ReturnValueMap::ScalarDeletingDtor | confirmed | Virtual slot 0 of ReturnValueMap::vftable with the scalar deleting shape: with ECX = this it calls the ReturnValueMap destructor body (014179e0), tests bit 0 o… |
| `01417e60` | ReturnValueMap::ReturnValueMap | confirmed | Fastcall with ECX = this. Stores PersistentBase::vftable, then PersistentComponent::vftable, then ReturnValueMap::vftable into this, and zeroes the words at th… |
| `01417a80` | ReturnValueMap::GetStaticDword | likely | Virtual slot 12 of ReturnValueMap::vftable. Returns the dword in the global DAT_01cbe254 (mov eax,[abs]; ret). |
| `01417a90` | ReturnValueMap::SetStaticDword | likely | Virtual slot 13 of ReturnValueMap::vftable. Stores its one stack argument into the global DAT_01cbe254 and returns with ret 4. |

**RevealState** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01460470` | RevealState::~RevealState | confirmed | Destructor body. Stores RevealState::vftable, frees the heap buffers at this+0x148 and this+0x108 when non-null (zeroing their pointers), calls 01300cd0 twice,… |
| `01460560` | RevealState::GetTypeName | confirmed | Virtual slot 11 of RevealState. Returns a pointer to the literal RevealState. |
| `01460570` | RevealState::GetClassDword | confirmed | Virtual slot 12 of RevealState. Returns the dword in the global DAT_01cbe568. |
| `01460580` | RevealState::SetClassDword | confirmed | Virtual slot 13 of RevealState. Stores its stack argument into DAT_01cbe568 and returns with ret 4. |
| `014605a0` | RevealState::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Calls the destructor body 01460470 on this; when bit 0 of the argument is set it frees this through the free thunk … |
| `01460790` | RevealState::Ctor | confirmed | Constructor. Calls the base constructor 0147c980 (CancellableStateMachineState), stores RevealState::vftable, zeroes the fields at +0xf8, +0xfc, +0x100, +0x108… |
| `014608a0` | RevealState::CtorWithArg | confirmed | Constructor overload that passes its argument to the base constructor 0147c9f0, stores RevealState::vftable, applies the same field defaults and name assignmen… |
| `01461b00` | RevealState::RestoreState | confirmed | Virtual slot 17 (restoreState). Switches on the phase at +0x118: case 2 allocates 0x18 bytes (00d833a6, 01417e60), sets the phase to 3 and re-enters slot 17; c… |
| `01460390` | RevealState::LeaveState | likely | Virtual slot 18 of RevealState. Loads ECX = this->mMachine ([this+8]), calls 013f39c0 on it, then returns 1. |
| `014609c0` | RevealState::Factory | likely | Allocates 0x15c bytes with operator new (00d833a6). A null allocation returns 0; otherwise it runs RevealState::Ctor (01460790) on the block and returns the ob… |

**SetEnvironmentNode** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0148fb30` | SetEnvironmentNode::TypeName | confirmed | No arguments. Returns the literal "SetEnvironmentNode" (mov eax, 018f73d8; ret). |
| `0148fc70` | SetEnvironmentNode::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape (ret 4): calls the body 0148fb70 (ECX=this), then when bit 0 of the flag argument is set calls _free (00d82463 thunk) on this. |
| `0148fcb0` | SetEnvironmentNode::DoBuildTarget | confirmed | Virtual slot 21, thiscall, one stack arg (ctx). Traces 'SetEnvironmentNode::doBuildTarget %d' with the play-area enum at this+0x10 and switches on it: 1 Target… |
| `0148eb80` | SetEnvironmentNode::SetPlayAreaElement | likely | Virtual slot 23, thiscall, two stack args (element object, env). Switches on the play-area enum at this+0x10: 0x30, 0x11, 1, 2, 0x37, 0x422 and 0x423 each pick… |
| `0148faa0` | SetEnvironmentNode::Ctor | likely | Thiscall. Calls the base constructor 0149ab30 (ECX=this), installs the SetEnvironmentNode vtable (018f7374), makes the string at +0x14 empty (size +0x28 = 0, c… |
| `0148fb40` | SetEnvironmentNode::GetStaticDword | likely | No arguments. Returns the dword global DAT_01cbe5e4. |
| `0148fb50` | SetEnvironmentNode::SetStaticDword | likely | One stack argument, ret 4. Stores the argument into the dword global DAT_01cbe5e4. |
| `0148fb70` | SetEnvironmentNode::DtorBody | likely | Non-freeing destructor body: installs the SetEnvironmentNode vtable, frees the heap buffer at +0x18 through the free thunk when the capacity at +0x2c is at lea… |
| `0148fbf0` | SetEnvironmentNode::Factory | likely | Calls operator new with 0x38 bytes (00d833a6); when non-null calls the constructor 0148faa0 on it and returns the object, otherwise returns null. |
| `0148fc50` | SetEnvironmentNode::RegisterFactory | likely | Pushes the factory 0148fbf0 and the classID 0x4a (74), calls 013ce130, then calls 013ce230 with its result in ECX. Returns. |
| `014911b0` | SetEnvironmentNode::Execute | likely | Virtual slot 15, thiscall, two stack args (list or out object, env). Traces '(gvt:SetEnvironmentNode' and branches on the type code at +0xc of the first child … |

**SoeUtil::List<TradingCardGame::Manager::WindowInfo,-1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010d1a70` | ListDtorBody | confirmed | Fastcall destructor body: stores the List<WindowInfo> vftable (01887964) at this, then calls 010d1570 (RectNodeListClear) on this, which unlinks and releases e… |
| `010d1a80` | ListScalarDeletingDtor | confirmed | Slot [0] of the List<WindowInfo> vftable with the scalar deleting destructor shape: stores the vftable at this, calls 010d1570 (RectNodeListClear) on this, the… |
| `010d0d50` | ListFreeBuffer4 | likely | Virtual method at slot [3] of SoeUtil::List<TradingCardGame::Manager::WindowInfo,-1>::vftable (01887964). Calls 00401c00 (FreeMemory) on its first stack argume… |

**SoeUtil::List<TradingCardGame::Manager::WindowInfo,256>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010d25d0` | ListFixed256ScalarDeletingDtor | confirmed | Slot [0] of the List<WindowInfo,256> vftable with the scalar deleting destructor shape: calls 010d2500 (ListFixed256DtorBody) on this, then if bit 0 of param_2… |
| `010d2480` | ListFixed256Ctor | likely | Fastcall with ECX=this. Stores the List<WindowInfo,-1> vftable at this, zeroes the dwords at this+4, this+8 and this+0xc, stores the List<WindowInfo,256> vftab… |
| `010d2500` | ListFixed256DtorBody | likely | Destructor body: stores the List<WindowInfo,256> vftable, calls 010d1570 (RectNodeListClear) on this, sets the dword at this+0x10 and this+0x14 to zero after r… |

**SubtractExpression** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0148e860` | SubtractExpression::TypeName | confirmed | No arguments. Returns the literal "SubtractExpression" (mov eax, 018f6ef4; ret). |
| `0148e8f0` | SubtractExpression::AssertFalseSlot15 | confirmed | Virtual slot 15, two stack args that it ignores (ret 8). Calls 012f5a60 with the expression "false", the file name ..\common\common\node\SubtractExpression.cpp… |
| `0148e910` | SubtractExpression::Evaluate | confirmed | Virtual slot 14, thiscall, two stack args (out ValueData, env). Traces '(Subtract', then evaluates the left child at this+0x1c and the right child at this+0x20… |
| `0148eaf0` | SubtractExpression::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape (ret 4): installs the SubtractExpression vtable, calls the base destructor body 0149ae80 (ECX=this), then when bit 0 of the fl… |
| `0148e7e0` | SubtractExpression::Ctor | likely | Thiscall. Calls the base constructor 0149adc0 (ECX=this), then installs the SubtractExpression vtable (018f6ea4) and returns this. Sets no fields of its own. |
| `0148e870` | SubtractExpression::GetStaticDword | likely | No arguments. Returns the dword global DAT_01cbe5e0 (mov eax, [01cbe5e0]; ret). |
| `0148e880` | SubtractExpression::SetStaticDword | likely | One stack argument, ret 4. Stores the argument into the dword global DAT_01cbe5e0. |
| `0148e8a0` | SubtractExpression::DtorBody | likely | Non-freeing destructor body: installs the SubtractExpression vtable (018f6ea4), calls the base destructor body 0149ae80 (ECX=this) and returns. It does not cal… |
| `0148ea70` | SubtractExpression::Factory | likely | Calls operator new with 0x24 bytes (00d833a6); when non-null calls the constructor 0148e7e0 on it and returns the object, otherwise returns null. |
| `0148ead0` | SubtractExpression::RegisterFactory | likely | Pushes the factory 0148ea70 and the classID 0x25 (37), calls 013ce130, then calls 013ce230 with its result in ECX. Returns. |

**SumExpression** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0148e030` | SumExpression::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape (ret 4): installs the SumExpression vtable, calls the base destructor body 0149ae80 (ECX=this), then when bit 0 of the flag ar… |
| `0148e090` | SumExpression::Evaluate | confirmed | Virtual slot 14, thiscall, two stack args (out ValueData, env). Traces 'initialcards: ', calls vtable slot 20 (SumExpression::GetCardList) for the card list, r… |
| `0148dfb0` | SumExpression::Factory | likely | Calls operator new with 0x2c bytes (00d833a6); when the pointer is non-null calls the constructor 0148dcd0 on it and returns the object, otherwise returns null. |
| `0148e010` | SumExpression::RegisterFactory | likely | Pushes the factory 0148dfb0 and the classID 0x170 (368), calls 013ce130, then calls 013ce230 with the result in ECX. Returns. |

**TournamentServiceBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013ea860` | TournamentServiceBase::~TournamentServiceBase | likely | Stores the TournamentServiceBase vftable, releases the std::string member at +0x60 (buffer freed when capacity exceeds 15, then reset to empty at +0x74), calls… |
| `013ea9b0` | TournamentServiceBase::TournamentServiceBase | likely | Constructor body: calls 01230d10, stores the TournamentServiceBase vftable at *this, then runs member constructors 005258fb, 013e7cf0 (twice), 013e8bf0 and 013… |
| `013eaa70` | TournamentServiceBase::ScalarDeletingDtor | likely | Thiscall with one stack flag: calls the destructor body 013ea860, frees this when flag bit 0 is set, and returns this. |

**TradingCard::Lobby** (16)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01301f80` | TradingCard::Lobby::Thunk_m4_Slot15 | confirmed | Adjustor thunk for a secondary vtable slot: subtracts 4 from ECX, loads the adjusted object's vtable and jumps to its slot at +0x3c (slot 15) with the same sta… |
| `01301f90` | TradingCard::Lobby::Vslot17_LazyChild2CAdd | confirmed | Virtual slot 17 (thiscall, two stack arguments): if the dword at this+0x2c is null, allocates 0x10 bytes through 00d833a6 and constructs the object with 013129… |
| `01302010` | TradingCard::Lobby::Vslot19_ForwardIf2C | confirmed | Virtual slot 19: returns 0 when the dword at this+0x2c is null; otherwise tail-calls 013110e0 with that object as ECX and returns its result (byte). |
| `01302030` | TradingCard::Lobby::Vslot21_ForwardIf2C | confirmed | Virtual slot 21: returns 0 when the dword at this+0x2c is null; otherwise tail-calls 01311c50 with that object as ECX and returns its result (byte). |
| `01302050` | TradingCard::Lobby::Vslot23_ChildByIndex | confirmed | Virtual slot 23 (thiscall, one stack argument): for an index 1 to 5 returns the dword at this+0x2c, +0x30, +0x34, +0x38 or +0x3c; returns 0 for any other index. |
| `013020a0` | TradingCard::Lobby::Vslot22_GetOrCreateChildByIndex | confirmed | Virtual slot 22 (thiscall, one stack argument): reads the child pointer for index 1 to 5 (fields +0x2c to +0x3c); when it is null, allocates 0x10 bytes through… |
| `013021f0` | TradingCard::Lobby::Vslot14_CallSlot23ThenForward | confirmed | Virtual slot 14 (thiscall, two stack arguments, ret 0xc): calls the object's own virtual slot 23 (+0x5c) with the first argument; if that returns non-null, cal… |
| `01308910` | TradingCard::Lobby::GetParamByKey | confirmed | Thiscall (this in ECX), stack argument = key, out ValueData pointer as the second stack argument. Virtual slot 15 (the keyed getter). Returns at once when this… |
| `0130b6a0` | TradingCard::Lobby::Dtor | confirmed | Destructor body. Stores TradingCard::Lobby::vftable at *this and this+4, releases the owned objects at +0x2c, +0x30, +0x34, +0x38, +0x3c and +0x54 through vtab… |
| `0130b860` | TradingCard::Lobby::TypeName | confirmed | Returns the string literal "Lobby" (mov eax,0x18c9e64; ret). |
| `0130b870` | TradingCard::Lobby::GetClassDword | confirmed | Returns the global dword DAT_01cbda64 (return DAT_01cbda64;). |
| `0130b880` | TradingCard::Lobby::SetClassDword | confirmed | Stores its dword argument into DAT_01cbda64 (DAT_01cbda64 = param_1;). |
| `0130b8a0` | TradingCard::Lobby::SecondaryDtorThunk | confirmed | Adjustor thunk (sub ecx,4; jmp 0130be50): converts the secondary-base this pointer to the primary and jumps to the slot-0 body 0130be50. |
| `0130bd70` | TradingCard::Lobby::Ctor | confirmed | Fastcall constructor. Stores PersistentBase::vftable and then PersistentComponent::vftable at *this, calls base initializer 01402960 on EBX, stores TradingCard… |
| `0130be50` | TradingCard::Lobby::ScalarDeletingDtor | confirmed | Scalar deleting destructor (bytes: push esi; mov esi,ecx; call 0130b6a0; test byte [esp+8],1; free when set; ret 4). Runs the TradingCard::Lobby dtor body, fre… |
| `0130bff0` | TradingCardLobbyNew | likely | Allocates 0x70 bytes with operator new (00d833a6). When the allocation is non-null it calls 0130bd70 (TradingCard::Lobby::Ctor) on the new object and returns i… |

**(no class)** (148)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01301ef0` | LobbySetContainingGroupID | confirmed | Thiscall (ECX = lobby object, stack arg = group id): when the dword at +0x14 is already nonzero, reports the assert 'mContainingGroupID == 0' with Lobby.cpp li… |
| `0131a340` | DoStartOfGameInit | confirmed | Thiscall with one int argument (no return value). When the object at this+0x410 is non-null it writes the trace text ZPMainController::doStartOfGameInit. If th… |
| `01354020` | CardInstanceDataCheck | confirmed | Calls 01410e90 and passes its result to 01411430; if the dword at param_2+0x64 is zero, logs Instace data was null for card with the values from 0138a380 and 0… |
| `013547e0` | AddLeagueCardAction | confirmed | Logs addLeagueCardAction with four integer arguments through 013508b0 and writes the same text to the game log via 013541f0; then calls 012b8f80 and 012b9be0 w… |
| `01354840` | ForEachCardCall0138a670 | confirmed | Sets uVar2 to 013f13f0() when the dword at this+0x32c is non-zero; walks the circular list at this+0x84 and, for each element at node+0xc, calls the dynamic-ca… |
| `0136d810` | HandleCaughtGameError | confirmed | Thiscall with a player argument. Calls 01354470 and tests 013ef1d0. When the test fails it logs "Player %d had an error, continuing the game." with the player … |
| `013881b0` | SetTargetFromPlayAreaTarget | confirmed | Thiscall with a target code (param_2) and a target id (param_3). Logs 'SetTargetFromPlayAreaTarget: %d, %d'. Code 1 stores the id at this+0xc, code 2 at this+0… |
| `013aa220` | DeckValidateMessage | confirmed | If this+0x0c is null it sets the bool at arg1+0x18 to 0 and assigns Invalid Deck! (13 chars, 0042131b) to the std::string at arg1+0x1c. Otherwise it calls 013c… |
| `01447da0` | TcgEventSessionManagerName | confirmed | Builds the std::string result in the hidden buffer param_1 for the session kind param_2: 1 gives Tcg_eventSession.managerSession, 2 gives Tcg_eventSession.mana… |
| `01448060` | StubTextGetStartTimeText | confirmed | Returns a std::string holding the literal TODO: getStartTimeText (22 bytes) through 0042131b. |
| `014480d0` | StubTextGetEndTimeText | confirmed | Returns a std::string holding the literal TODO: getEndTimeText (20 bytes) through 0042131b. |
| `01448140` | StubTextGetRemainingTimeText | confirmed | Returns a std::string holding the literal TODO: getRemainingTimeText (26 bytes) through 0042131b. |
| `00ad1220` | PollStateUntil4OrZero | likely | Repeatedly reads state at +0x2ba90; returns 1 at state 4, returns 0 at state 0, otherwise calls vtable slot +0x5c and sleeps 25 ms before retrying. |
| `00ad14f0` | SetTcgStateAndTimestamp | likely | Stores the supplied state at +0x2ba90, reads the current 64-bit clock, and stores the clock pair at +0x5ef50/+0x5ef54. |
| `00ad1530` | ElapsedSinceTcgStateTimestamp | likely | Reads the current 64-bit clock and returns its low dword minus the stored low dword at +0x5ef50. |
| `00f9d320` | CardClientInit | likely | Thiscall with the client object and about twelve stack arguments: asserts 'iface != 0' against '.\cardclient.cc' line 0xae when the interface pointer is null, … |
| `010d25f0` | TcgManagerSingletonCtor | likely | Fastcall with ECX=this (the owner object). Constructs its subobjects: StringFixed256 at the start (004029b0), eight fixed strings through 00703540, two SoeUtil… |
| `010d27a0` | TcgManagerSingletonDtorBody | likely | Destructor body of the object constructed by 010d25f0 (ECX=this): calls 010cffb0 (ResetFlaggedState3B8), clears the global DAT_01cb9638, destroys the List<Wind… |
| `01259d10` | TcgInvoke_CardQueried | likely | Gated on param_1 != 0 and on 0131bd10(param_2) != 0. Builds the Scaleform action name CardQueried through 012ec220, fills its argument object via 0131e290, 013… |
| `01259ea0` | TcgBuildEffectAction | likely | Allocates 0x40 bytes through 00d833a6 and creates either AddEffectToTarget (when 01257ed0(param_2, 0x1b3b8, 0) == 0x1adb6) or EffectPlayAnimation through 012ec… |
| `0125a4e0` | TcgBuildButtonArgs | likely | Builds a TCG button argument object: a string taken from the 01322040 result is copied through 013241b0 into a local. Then sets ElementID from param_2 (0130e7b… |
| `0125abb0` | TcgInvoke_UpdateModifierArrays | likely | Gated on 0124d010() != 0. Builds the Scaleform action UpdateModifierArrays through 012ec220 and dispatches it through 012e5730 and 012e5820. Returns nothing. |
| `0125ac70` | TcgInvoke_AddModifier_CantPlayColor | likely | Gated on 0124d010() != 0. Calls 01259110 (map find) on the tree whose head is at +0x14 and calls 00d83c2d on a bad find result. Builds the AddModifier_CantPlay… |
| `0125ae30` | TcgInvoke_AddModifier_CardColor | likely | Same structure as 0125ac70 (map find through 01259110, owner check, myPlayer, usePoint and modifierColor argument mapping) but invokes AddModifier_CardColor th… |
| `0125aff0` | TcgInvoke_AddModifier_CardValue | likely | Gated on 0124d010() != 0 and the 01259110 map find. Builds a NewModifier object (myPlayer from 013809c0, usePoint, modifierNumber from param_2, modifierColor m… |
| `0125b470` | TcgInvoke_AddModifier_LowNumberWins | likely | Gated on 0124d010() != 0 and the 01259110 map find. Builds a NewModifier sub-object (myPlayer, usePoint mapping thisFight/nextFight/nextDuel/nextBaseAttack fro… |
| `0125ba30` | TcgInvoke_ShowBattleWin | likely | Gated on 0124d010() != 0. Calls 0131de60 and 0131e290 for the battle result, builds the Scaleform action ShowBattleWin through 012ec220, sets DAT_018cd818 thro… |
| `0125bfd0` | TcgInvoke_EndOfRound | likely | Builds the Scaleform action EndOfRound through 012ec220 and dispatches it through 012e5730 and 012e5820. Returns nothing. |
| `0125c080` | TcgInvoke_ActivateDestroyBaseCard | likely | Gated on 0124d010() != 0 and the 01259110 map find. Builds a BaseCardData sub-object (myBase boolean from 013809c0 owner compare, targetElementID from 01380990… |
| `0125c380` | TcgInvoke_HideBaseCard | likely | Same structure as 0125c080 (BaseCardData with myBase and targetElementID), invokes HideBaseCard through 012ec220, sets BaseCardToHide through 012ebe00, and dis… |
| `0125ca60` | TcgInvoke_FlipBaseCard | likely | Gated on 0124d010() != 0 and the 01259110 map find. Builds a BaseCardData sub-object (myBase, targetElementID, faceDown boolean from param_2), invokes FlipBase… |
| `0125cdf0` | TcgInvoke_SwapDuelCardWithHandCard | likely | Builds the Scaleform action SwapDuelCardWithHandCard through 012ec220 with Elements sub-objects from 0131e290 and the 01259110 map finds; sets DuelCard and Han… |
| `01262d60` | BuildReplaceBasesAction | likely | Looks up an entry keyed by the int argument (FUN_01259110 on the object's +0x14 tree) and, when the stored value differs from the object's +0x18 end, walks zon… |
| `01263140` | SetCurrentPlayer | likely | Gets the game object (0124d010) and its zone collection; when the int argument differs from the stored current player at +0x34, stores it and queues StartPlaye… |
| `012638a0` | UpdateCurrentTurn | likely | Gets the game object and zone collection; when the int argument differs from the stored current player at +0x34, stores it and queues StartPlayerTwoTurn or Sta… |
| `012648b0` | BuildInitGameScreen | likely | Gets the game object and the zone collection (vtable +0x144, +0x34 walk); for zone indices 0 to 3 it pushes every object whose vtable +0x15c returns 0x222e3 an… |
| `012d94e0` | FormatPairErrorReport | likely | Walks the tree whose proxy is this+0x30 (head at this+0x34). For each node whose +0x10 value is not 3 it formats a '%d -> %d:' line from the node's +0xc and +0… |
| `012da220` | ClearTreeNodeVectors | likely | Walks five trees (proxies at this+4, +0x10, +0x1c, +0x28, +0x34): for each node it sets the vector at +0x14/+0x18 to empty (for the +0x28 tree it frees each 0x… |
| `012dc150` | IntMapAppendValueAt10 | likely | Same operation as 012dc050 on the int-keyed map whose proxy is this+0xc and header at this+0x10: finds the key, creates the entry through 012dbc60 when missing… |
| `012dc250` | IntMapAppendValueAt1c | likely | Same operation as 012dc050 on the int-keyed map whose proxy is this+0x18 and header at this+0x1c: finds the key, creates the entry through 012dbc60 when missin… |
| `012dc350` | ArchetypeGroupIntKey | likely | For each element of the vector at this+4..+8 that dynamic_casts (00d8d382) to Archetype from Sortable, takes a ValueData from the archetype via 012fa910 and ca… |
| `012dc4b0` | ArchetypeGroupStringKey | likely | As 012dc350 for the string-keyed map: Archetype items whose ValueData type tag is 3 are added to the entry from 012dbd60 (string key), others go to the list th… |
| `012dc600` | DeckBuilderCollectionCount | likely | Walks the vector at param_2 in pairs, calls the predicate in vtable slot 0 of param_1 on each entry and counts matches in a keyed map through 0051ff49; then wa… |
| `012dd0f0` | CheckCardsOnlyFromCollection | likely | Groups Archetype items with 012dc350 (argument 0x49), counts entries for collection param_3, and when the count is non-zero formats 'You may only have cards fr… |
| `012ef500` | TimedQueueTick | likely | Thiscall with one float dt (ECX=this, EH frame). Subtracts dt from each float in the vector at this+0x80..0x84 and removes entries that drop to 0 or below (cou… |
| `0130c800` | LobbyAddMemberToRole | likely | Thiscall role insert. Asserts the role argument is valid (assert text "kInvalidRole != role", file Lobby.cpp line 0x25b). Loops while the role matches: it runs… |
| `01315010` | SetStringField420 | likely | Thiscall with one string argument (ret 4): assigns the argument into this+0x420 with basic_string::assign (00420ff1, pos 0, count npos). |
| `01315030` | SetStringField43c | likely | Thiscall with one string argument (ret 4): assigns the argument into this+0x43c with basic_string::assign (00420ff1, pos 0, count npos). |
| `01315d30` | ValueData::StoreHolder | likely | Thiscall with two stack arguments (ret 8). Calls 01300680(0xe) on this. Asserts the second argument is non-null (ValueData.h line 0x1e8, message value). Create… |
| `013224b0` | ValidatePlayActionCard | likely | Thiscall (param_1 = game object, param_2 = card/action object). Returns 1 at once when param_2 vtable slot [0x1e8] is true. Otherwise reads phase (game vtable … |
| `0132e1d0` | DynCastField3a8ToCard | likely | Fastcall (ECX = this): calls 01357a00 with the dword at this+0x3a8 and passes the result to 00d8d382 as a dynamic_cast from PlayElement (PlayElement::RTTI_Type… |
| `01330220` | UnwindDestroyMemberAt54 | likely | Cleanup routine: computes this+0x54, calls 0132e820 on that sub-object, then tail-jumps to 0126c560 with the same ECX. It is the cleanup path of an object whos… |
| `01333767` | TraceEndingGame | likely | Calls 013c26e0 (a no-op trace; the 'Ending Game' text is not emitted) and then the object's virtual slot 3 (+0xc) through the ESI register, then returns after … |
| `01335f20` | DoReshuffleFor | likely | Logs 'CWGame::doReshuffleFor( %d )' and looks up the player id (01350660). When found and callee 0139d7a0 (+0x48) is non-zero it gets the object at 0139d7b0 (+… |
| `013395b0` | PruneModifierIdsList0x3b0 | likely | For each id in the vector at this+0x3b0..+0x3b4: resolves the object (01357a00) and dynamic_casts it from PlayElement to AttributeModifier (00d8d382). When the… |
| `01339820` | PruneModifierIdsList0x3c0 | likely | Same routine as PruneModifierIdsList0x3b0 over the vector at this+0x3c0..+0x3c4: resolves ids, dynamic_casts to AttributeModifier, collects those whose owner (… |
| `01363f00` | RemoveCommandFromCommandDurationMap | likely | Traces 'Game::removeCommandFromCommandDurationMap %d (%p)' when the argument is non-null. Walks every node of the command map (header at this+0x10c) and, in ea… |
| `01365140` | PushIdOfValidActionFilter | likely | Asserts that its argument is non-null (assert text 'validActionFilter', Game.cpp line 0x1a90), then appends the argument's id (01340370, the dword at +0x34) to… |
| `01365390` | SuppressCardGameText | likely | When the card argument is non-null: traces 'Suppresing game text of card: %d' with the card's id (01340370, the dword at +0x34), then appends that id to the 4-… |
| `0136d4f0` | SubmitListEntriesOrReject | likely | Thiscall with a command object param_2. When the byte at this+0x362 is clear, it submits each entry of the two 4-byte vectors at this+0x28 and this+0x48 throug… |
| `0136d640` | DispatchCommandObject | likely | Thiscall with two arguments and a command object param_3. When this+0x362 is set it logs "CW_ERROR: Server ignoring game commands!" and deletes param_3 through… |
| `0136d6a0` | SubmitListEntriesIfActive | likely | Thiscall with a command object param_2. When the byte at this+0x362 is clear and the byte at this+0x388 is clear, it calls param_2's slot 0 with no argument an… |
| `01371870` | GetOrCreateCardForID | likely | Thiscall (receiver fields +0x13c, +0x14d are read). Calls 01357a00 with param_2 to find the card for that element id. When the lookup fails it builds the messa… |
| `01371b10` | GetOrCreateCardForForcedID | likely | Same shape as GetOrCreateCardForID: 01357a00 lookup; on failure logs 'Couldn't get card from element may in getOrCreateCardForForcedID!' and 'Card: %d, player:… |
| `013860d0` | IdVectorToCommandObjects_0x20 | likely | Thiscall with a zeroed output vector (param_2). Walks the 4-byte id vector whose begin/end are at this+0x20/+0x24. For each id it calls 01357a00 to get the obj… |
| `013861d0` | IdVectorToCommandObjects_0x30 | likely | Same as 013860d0 but walks the id vector at this+0x30/+0x34 (-0x2c guard). Each id is resolved with 01357a00, dynamic_cast from PlayElement to CommandObject (0… |
| `013868d0` | PropertyMap_ValueForKey | likely | Thiscall with ECX = the property map (environment +0x7c) and one stack argument (pointer to a key). It builds a temporary ValueData (012fa910), copies the key … |
| `013922d0` | RemoveCardAction | likely | Asserts the action argument is non-null (assert text 'action' at Card.cpp line 0xcfa). Takes the action's id with 01321f20, erases the first matching 4-byte en… |
| `01394b10` | GetActionById | likely | Fastcall on the card receiver with one stack argument (action id). Fills a temporary list through the receiver's vtable slot 50 (+0xc8) and returns the first e… |
| `01399a30` | IntegerListKeywordIdsCollect | likely | Zeroes the 3-word vector at param_2 (+4,+8,+0xc), then calls virtual slot +4 of param_1 with (2, &local) and, when it returns true, walks the ValueData integer… |
| `0139e9a0` | PlayArea::CountTrueBooleanEntries | likely | Thiscall. Fills a temporary list through the object's vslot +0x34 (slot 13), then for each element calls its vslot +4 with key 0x2ea and a ValueData output; co… |
| `0139f550` | FirstKey3ElementAsCard | likely | Thiscall (ECX = object). Looks up map key 3 at object+0xc via 01343ed0. If that vector is non-empty it dynamic-casts its first element from PlayElement to Card… |
| `0139f5d0` | TryFirstKey3Card | likely | Bool method (ECX = object, arg = out pointer). Same lookup as 0139f550: map key 3 at object+0xc via 01343ed0, dynamic cast of the first element from PlayElemen… |
| `013d5810` | CollectionItemFactory | likely | Allocates 0x30 bytes with 00d833a6 (operator new); when the allocation is nonzero it constructs the object through 013d55b0 (CollectionItem constructor) and re… |
| `013d5870` | CollectionItemRegisterFactory | likely | Registers the CollectionItem factory 013d5810 under key 0x34: calls 013ce130 with (0x34, factory) and 013ce230 with the same key and factory, then returns. |
| `013eb7b0` | ValidateMinDrawCount | likely | Calls vtable slot +0x48 on its first argument to get a container whose begin/end (+4/+8) bracket 8-byte entries, counts the entries, and when the count is belo… |
| `013ec860` | CheckCardsFromSetRule | likely | Collects a card-list through vtable slot +0x54 of its first argument and iterates a collection of 0x48-byte entries via 012dc350 and 012d6290; for each entry w… |
| `013ecbe0` | CheckCopyLimitPerTitle | likely | Takes the copy limit (param_1) and collects card ids through vtable slot +0x54 and 012dc4b0; counts entries through 012d8da0. When the count is non-zero it for… |
| `013ecfe0` | CheckCommonOnlyRule | likely | Runs the 40-card and 3-copy checks (013eb7b0, 013ecbe0), then walks a 0x4c-byte-entry collection through 012dc350 and 012d6290, calling 012dc250 per card id of… |
| `013ed2b0` | CheckNoRareRule | likely | Runs the 40-card and 3-copy checks (013eb7b0, 013ecbe0), then walks the same 0x4c-byte-entry collection, calling 012dc250 per card id of each entry whose tag (… |
| `01413010` | SetupAndPushResponseState | likely | Sets this+0x11c to 2, calls FUN_0142a6d0 and FUN_0142a6b0 setup helpers, and reads the environment player through FUN_01383890/FUN_01383810: when a player is s… |
| `01418450` | CastHandle0x60ToCWCard | likely | Fastcall with ECX = object. Reads the dword at object+0x60, resolves it through FUN_01357a00, and returns the result of FUN_00d8d382 (a dynamic_cast from PlayE… |
| `01448650` | TcgEventRewardText | likely | Builds the TCG event-session reward text: a switch over the reward kind (cases 1 to 9, with default), keyed by Tcg_eventSession.reward* strings (participant, p… |
| `01452900` | CreateCWArchetype | likely | Pushes 0x34 and calls allocator FUN_00d833a6 (one stack argument); if the pointer is non-null, calls ctor FUN_01452730 with it in ECX and returns that object, … |
| `01452960` | RegisterFactory0x222f4 | likely | Pushes the key 0x222f4 and the address of FUN_01452900, calls FUN_013ce130, moves its return value into ECX and calls FUN_013ce230; same registration shape as … |
| `014536e0` | CreateCWValueNode | likely | Pushes 0x2c and calls allocator FUN_00d833a6 (one stack argument); if the pointer is non-null, calls ctor FUN_014534f0 with it in ECX and returns that object, … |
| `01453740` | RegisterFactory0x222fa | likely | Pushes the key 0x222fa and the address of FUN_014536e0, calls FUN_013ce130, moves its return value into ECX and calls FUN_013ce230; same registration shape as … |
| `01453970` | CreateCWTargetQueryNode | likely | Pushes 0x34 and calls allocator FUN_00d833a6 (one stack argument); if the pointer is non-null, calls ctor FUN_014537c0 with it in ECX and returns that object, … |
| `014539d0` | RegisterFactory0x222f5 | likely | Pushes the key 0x222f5 and the address of FUN_01453970, calls FUN_013ce130, moves its return value into ECX and calls FUN_013ce230; same registration shape as … |
| `01454280` | CreateCWSumExpression | likely | Pushes 0x2c and calls allocator FUN_00d833a6 (one stack argument); if the pointer is non-null, calls ctor FUN_014540e0 with it in ECX and returns that object, … |
| `014542e0` | RegisterFactory0x222f2 | likely | Pushes the key 0x222f2 and the address of FUN_01454280, calls FUN_013ce130, moves its return value into ECX and calls FUN_013ce230; same registration shape as … |
| `01460a30` | RegisterFactory_0xff | likely | Static initializer: calls 013ce130 with classID 0xff and factory 014609c0, then calls 013ce230 on the returned registry slot to store the factory under 0xff. |
| `01460a50` | SetRevealedCards | likely | Appends the int elements of the source vector (begin at param_2+4, end at +8) to the list at this+0x148 (begin) .. +0x14c (end), growing through 012619f0 when … |
| `01464310` | CreateReshuffleState | likely | Allocates 0x114 bytes with operator new (00d833a6). If the allocation is non-null, it runs the ReshuffleState constructor 01463c00 on the block and returns the… |
| `01464380` | RegisterReshuffleStateFactory | likely | Pushes the factory address 01464310 and the constant 0x136 into a registry: it calls 013ce130 to obtain the registry object and then 013ce230 with (0x136, 0146… |
| `01464f20` | CreateProcessEventEmissionState | likely | Allocates 0x148 bytes with operator new (00d833a6). If the allocation is non-null it runs the no-argument constructor 01464d40 on it and returns the object; ot… |
| `01464f90` | RegisterProcessEventEmissionStateFactory | likely | Pushes the factory address 01464f20 and the constant 0xfc into the registry: calls 013ce130 to obtain the registry object, then 013ce230 with (0xfc, 01464f20),… |
| `014670b0` | CreatePlaySingleCardState | likely | Allocates 0x108 bytes with operator new (00d833a6). If the allocation is non-null it runs the no-argument constructor 01466730 on the block and returns the obj… |
| `01467120` | RegisterPlaySingleCardStateFactory | likely | Pushes the factory address 014670b0 and the constant 0x14b into the registry: calls 013ce130 to obtain the registry object, then 013ce230 with (0x14b, 014670b0… |
| `01468570` | CreatePlayCardState | likely | Allocates 0xfc bytes with operator new (00d833a6). If the allocation is non-null it runs the no-argument constructor 01467b80 on the block and returns the obje… |
| `014685e0` | RegisterPlayCardStateFactory | likely | Pushes the factory address 01468570 and the constant 0xf7 into the registry: calls 013ce130 to obtain the registry object, then 013ce230 with (0xf7, 01468570),… |
| `01472830` | PrefAutoSkipCheck | likely | Called by 01474190 (DoublePass loop). Asserts this+0x108 (mCurrentPlayer) is nonzero (DoublePassState.cpp:0x1ec), gets the current account id (01344e60), looks… |
| `0147b220` | LoadStartReturnValues | likely | Fastcall (ECX = this). Asserts the returnValueMap (013f37e0 must be non-null) and hasIntegerKey(kPlayerReturn) (key 4), reads the player with 01417170(4) and r… |
| `014e4c60` | RegisterFactory0x16b | likely | Pushes the address of 014e4c00 and key 0x16b (decimal 363), calls 013ce130, then calls 013ce230 with 013ce130's return value in ECX (the same registration pair… |
| `01503e80` | FindAlternateDiscardArea | likely | Thiscall: calls 01383c10 and 01383e60; when the 01383e60 result is 0 or equals [this+0x64] and 01383c10 is nonzero, calls 01322070 and, when it is nonzero, log… |
| `01503ef0` | FindAlternateDiscardPosition | likely | Thiscall: calls 01383c10; when it is nonzero, calls 013220a0, logs Found alternate discard area Position! %d through 013846b0 into this+0x84 and returns the 01… |
| `01505d50` | CreateCommandObject_LoseGame | likely | Allocates 0x128 bytes with operator new (00d833a6); if the allocation is non-null it runs the LoseGame constructor 01505c10 on it and returns that object plus … |
| `01505dd0` | RegisterCommandObject_LoseGameFactory | likely | Registers the factory 01505d50 under key 0x181: calls 013ce130 with the key and the factory, then calls 013ce230 on the returned registry object (ECX). |
| `015062f0` | CreateCommandObject_Kill | likely | Allocates 300 bytes with operator new (00d833a6); if non-null runs the default ctor 015061a0 and returns the object plus 8, otherwise returns 0. |
| `01506370` | RegisterCommandObject_KillFactory | likely | Registers factory 015062f0 under key 0xbe: calls 013ce130(0xbe, factory), then calls 013ce230 on the returned registry object. |
| `015066d0` | CreateCommandObject_IntroduceCard | likely | Allocates 0x128 bytes with operator new (00d833a6); if non-null runs the default ctor 01506550 and returns the object plus 8, otherwise returns 0. |
| `01506750` | RegisterCommandObject_IntroduceCardFactory | likely | Registers factory 015066d0 under key 0x173: calls 013ce130(0x173, factory), then calls 013ce230 on the returned registry object. |
| `01506ee0` | CreateCommandObject_InstallActionMap | likely | Allocates 0x140 bytes with operator new (00d833a6); if non-null runs the default ctor 01506d40 and returns the object plus 8, otherwise returns 0. |
| `01506f60` | RegisterCommandObject_InstallActionMapFactory | likely | Registers factory 01506ee0 under key 0xca: calls 013ce130(0xca, factory), then calls 013ce230 on the returned registry object. |
| `01507440` | CreateCommandObject_InstallActionFilter | likely | Allocates 300 bytes with operator new (00d833a6); if non-null runs the default ctor 01507280 and returns the object plus 8, otherwise returns 0. |
| `015074c0` | RegisterCommandObject_InstallActionFilterFactory | likely | Registers factory 01507440 under key 0xd4: calls 013ce130(0xd4, factory), then calls 013ce230 on the returned registry object. |
| `01507a20` | CreateCommandObject_InstallAction | likely | Allocates 0x128 bytes with operator new (00d833a6); if non-null runs the default ctor 01507930 and returns the object plus 8, otherwise returns 0. |
| `01507aa0` | RegisterCommandObject_InstallActionFactory | likely | Registers factory 01507a20 under key 0x171: calls 013ce130(0x171, factory), then calls 013ce230 on the returned registry object. |
| `015088b0` | CreateCommandObject_InfoBox | likely | Allocates 0x130 bytes with operator new (00d833a6); if non-null runs the default ctor 01507cc0 and returns the object plus 8, otherwise returns 0. |
| `01508930` | RegisterCommandObject_InfoBoxFactory | likely | Registers factory 015088b0 under key 0x18b: calls 013ce130(0x18b, factory), then calls 013ce230 on the returned registry object. |
| `01508c90` | CreateCommandObject_GetTopCard | likely | Allocates 0x128 bytes with operator new (00d833a6); if non-null runs the default ctor 01508b10 and returns the object plus 8, otherwise returns 0. |
| `01508d10` | RegisterCommandObject_GetTopCardFactory | likely | Registers factory 01508c90 under key 0x1b5: calls 013ce130(0x1b5, factory) and then 013ce230 on the returned registry object in ECX (the bytes show the second … |
| `015090f0` | CreateCommandObject_GetTarget | likely | Allocates 0x124 bytes with operator new (00d833a6); if non-null runs the default ctor 01508ee0 and returns the object plus 8, otherwise returns 0. |
| `01509170` | RegisterCommandObject_GetTargetFactory | likely | Registers factory 015090f0 under key 0xd6: calls 013ce130(0xd6, factory), then calls 013ce230 on the returned registry object. |
| `01509630` | CreateCommandObject_GameProperties | likely | Allocates 0x138 bytes with operator new (00d833a6); if non-null runs the default ctor 015094e0 and returns the object plus 8, otherwise returns 0. |
| `015096b0` | RegisterCommandObject_GamePropertiesFactory | likely | Registers factory 01509630 under key 0xd2: calls 013ce130(0xd2, factory), then calls 013ce230 on the returned registry object. |
| `01509af0` | CreateCommandObject_EmitSound | likely | Allocates 0x128 bytes with operator new (00d833a6); if non-null runs the default ctor 01509970 and returns the object plus 8, otherwise returns 0. |
| `01509b70` | RegisterCommandObject_EmitSoundFactory | likely | Registers factory 01509af0 under key 0x183: calls 013ce130(0x183, factory), then calls 013ce230 on the returned registry object. |
| `01509e70` | CreateCommandObject_DummyEvent | likely | Allocates 0x124 bytes with operator new (00d833a6); if non-null runs the default ctor 01509d40 and returns the object plus 8, otherwise returns 0. |
| `01509ef0` | RegisterCommandObject_DummyEventFactory | likely | Registers factory 01509e70 under key 0x149: calls 013ce130(0x149, factory), then calls 013ce230 on the returned registry object. |
| `0150a620` | CreateCommandObject_DrawCard | likely | Allocates 0x13c bytes with operator new (00d833a6); if non-null runs the default ctor 0150a3a0 and returns the object plus 8, otherwise returns 0. |
| `0150a6a0` | RegisterCommandObject_DrawCardFactory | likely | Registers factory 0150a620 under key 0xc1: calls 013ce130(0xc1, factory), then calls 013ce230 on the returned registry object. |
| `0150ad00` | CreateCommandObject_DiscardPile | likely | Allocates 0x134 bytes with operator new (00d833a6); if non-null runs the default ctor 0150aa10 and returns the object plus 8, otherwise returns 0. |
| `0150ad80` | RegisterCommandObject_DiscardPileFactory | likely | Registers factory 0150ad00 under key 0xdd: calls 013ce130(0xdd, factory) and then 013ce230 on the returned registry object. |
| `0150b390` | CreateCommandObject_Discard | likely | Calls operator new (00d833a6) with 0x140 bytes; if the allocation is non-null it calls the constructor 0150b2e0 on it and returns block+8, otherwise returns 0. |
| `0150c120` | CreateCommandObject_CustomChoiceDialog | likely | Calls operator new (00d833a6) with 0x124 bytes; if the block is non-null it calls the default constructor 0150bdf0 on it and returns block+8, otherwise returns… |
| `0150c800` | CreateCommandObject_CreateCard | likely | Calls operator new (00d833a6) with 0x130 bytes; if the block is non-null it calls the default constructor 0150c780 on it and returns block+8, otherwise returns… |
| `0150d100` | CreateCommandObject_Clone | likely | Calls operator new (00d833a6) with 0x128 bytes; if the block is non-null it calls the default constructor 0150cf70 on it and returns block+8, otherwise returns… |
| `0150d5c0` | CreateCommandObject_ChainedAction | likely | Calls operator new (00d833a6) with 0x124 bytes; if the block is non-null it calls the default constructor 0150d3d0 on it and returns block+8, otherwise returns… |
| `0150d930` | CreateCommandObject_CancelCommand | likely | Calls operator new (00d833a6) with 0x124 bytes; if the block is non-null it calls the default constructor 0150d820 on it and returns block+8, otherwise returns… |
| `0150dd00` | CreateCommandObject_Attach | likely | Calls operator new (00d833a6) with 0x124 bytes; if the block is non-null it calls the default constructor 0150dba0 on it and returns block+8, otherwise returns… |
| `0150e150` | CreateCommandObject_AlternateMultiAction | likely | Calls operator new (00d833a6) with 0x124 bytes; if the block is non-null it calls the default constructor 0150df50 on it and returns block+8, otherwise returns… |
| `0150ea00` | CreateCommandObject_AddToken | likely | Calls operator new (00d833a6) with 0x13c bytes; if the block is non-null it calls the default constructor 0150e770 on it and returns block+8, otherwise returns… |
| `0150ee50` | CreateCommandObject_AddSuppressEffect | likely | Calls operator new (00d833a6) with 0x130 bytes; if the block is non-null it calls the default constructor 0150ed10 on it and returns block+8, otherwise returns… |
| `0150f350` | CreateCommandObject_AddKeyword | likely | Calls operator new (00d833a6) with 0x130 bytes; if the block is non-null it calls the default constructor 0150f210 on it and returns block+8, otherwise returns… |
| `0150f730` | CreateCommandObject_AddGameText | likely | Calls operator new (00d833a6) with 0x124 bytes; if the block is non-null it calls the default constructor 0150f640 on it and returns block+8, otherwise returns… |
| `0150fca0` | CreateCommandObject_AddAttrModFilter | likely | Calls operator new (00d833a6) with 0x134 bytes; if the block is non-null it calls the default constructor 0150fb50 on it and returns block+8, otherwise returns… |
| `01510890` | CreateProcessEventEmissionStateForActivePlayer | likely | Calls operator new (00d833a6) with 0x148 bytes; if the block is non-null it calls the default constructor 015106b0 on it and returns that result (the object it… |

### `minigame.tcg.action`

**ActionNode** (28)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0149e640` | ActionNode::Ctor | confirmed | Calls base constructor 0149ab30 on the object, stores ActionNode::vftable at offset 0, zeroes the dwords at +0x10, +0x14, +0x18, +0x1c and +0x20, the byte at +… |
| `0149e710` | ActionNode::Dtor | confirmed | Stores ActionNode::vftable, then for the owned child at +0x1c and the child at +0x20 (if non-null) calls its virtual slot 0 with flag 1 (scalar deleting destru… |
| `0149e820` | ActionNode::GetNodeTarget | confirmed | Traces "ActionNode::getNodeTarget %d" with its first argument, then calls the virtual at +0x200 of the object returned by 01381380 (the environment's mGame fie… |
| `0149e880` | ActionNode::EvaluateTraced | confirmed | Traces "(ActionNode". If the Force byte at +0x24 is set it traces "Setting Force!" and calls 01387670. It then selects a virtual by the int at +0x10 (cases 1..… |
| `0149eea0` | ActionNode::ScalarDeletingDtor | confirmed | Calls destructor body 0149e710 and then frees the object with _free when bit 0 of the flag argument is set; returns the object pointer. |
| `0149ef40` | ActionNode::CheckTargetThis | confirmed | Traces "target == this", reads the list head at +8, then walks the (key at +0xc, value at +0x10) entries calling this->vtable[21](key, value, environment, 0) f… |
| `0149f030` | ActionNode::CheckTarget1 | confirmed | Traces "target == target1", resolves target1 with 01381460 (the result is null-tested; a null result traces "Got NULL target!" but does not return), then walks… |
| `0149f0f0` | ActionNode::CheckDynamicCard | confirmed | Traces "target == DyanmicCard", resolves the target with 01381460 and a dynamic-card lookup 013836f0 (traces "Couldn't get dynamicCard" when that is null), the… |
| `0149f1e0` | ActionNode::CheckTarget2 | confirmed | Traces "target == target2". When the node has no target2 (01381d40 == 0), the cost-check flag 01383b90 is 1 and 013815e0 returns 1, it traces the no-target2 co… |
| `0149f310` | ActionNode::CheckLastPlayedCard | confirmed | Traces "target == LastPlayedCard", resolves the last-played card with 01381460 and 01383e90 (returns 0 with "Last Played Card is NULL!!!" when it is null), the… |
| `0149f400` | ActionNode::CheckLastOwner | confirmed | Traces "target == LastOwner", resolves the last owner with 01381460 and 01383790 (traces "Last Owner NULL!!!!!" when null), traces "Last OwnerCard: " with the … |
| `0149f630` | ActionNode::CheckTargetOwner | confirmed | Traces "target == TargetOwner", resolves the target with 01381460, returns 0 with "Target is null!" when null, resolves a Card with a dynamic cast (01381460 wi… |
| `0149f7a0` | ActionNode::CheckTurnProperties | confirmed | Traces "target == turn properties". For each entry of the list at +8 it allocates 0x138 bytes (00d833a6), and when the allocation succeeds calls 01509570(key, … |
| `0149fa10` | ActionNode::CheckDrawDeck | confirmed | Traces "target == draw deck" and walks the entry list. Entries with key 0x15 end the walk, entries with key 7 or 8 are resolved with 01381460 and 013815a0 and … |
| `0149fcc0` | ActionNode::CheckOppDrawDeck | confirmed | Traces "target == Opp draw deck" and walks the entry list. Entries with keys 0x15, 7 and 8 select sub-branches; key 8 entries are resolved with 01381460 and 01… |
| `0149fec0` | ActionNode::CheckActivePlayer | confirmed | Traces "target == activePlayer", resolves the environment's mGame with 01381380 and calls its virtual +0x200 with key 0x3e and the environment to get the activ… |
| `0149ffa0` | ActionNode::CheckTarget3 | confirmed | Traces "target == target3", resolves target3 with 01381460 and 01383ec0(0x36) (traces "Couldn't get target 3!" and returns 0 when null), then walks the entry l… |
| `014a0090` | ActionNode::CheckTarget4 | confirmed | Traces "target == target4", resolves target4 with 01381460 and 01383ec0(0x37) (traces "Couldn't get target 4!" and returns 0 when null), then walks the entry l… |
| `014a0180` | ActionNode::CheckDefaultTarget | confirmed | Traces "target == defaultTarget", resolves the environment's mGame with 01381380 and calls its virtual +0x200 with the node's NodeTarget (+0x10) and the enviro… |
| `014a0280` | ActionNode::CheckOriginObj | confirmed | Traces "target == originobj", resolves the origin object with 01381460 and 01383ec0(0x38) (traces "Couldn't get originobj!" and returns 0 when null), then walk… |
| `014a0be0` | ActionNode::MappedAttributeMod | confirmed | Traces "target == MappedAttributeMod" and "AttributeMod: " with the int-value-map value of the first entry. Resolves the target value through vtable[20] (0x50)… |
| `014a2cb0` | ActionNode::AddPickListCommand | confirmed | Traces "Adding a PickList command", allocates an object with operator new and, when that succeeds, calls 014573f0 on this+4 and then 015028e0 with the second a… |
| `014a2d60` | ActionNode::AddChainedActionCommand | confirmed | Traces "Adding a ChainedAction command". Requires key 0x1c (kValueDataListKey) to be present and a value-data-list value (type 8); allocates 0x124 bytes and ca… |
| `014a39a0` | ActionNode::AddPushPhaseCommand | confirmed | Traces "Doing a PushPhase", requires key kPhaseKey to be an integer value (type 2), allocates an object with operator new and, when that succeeds, calls 014573… |
| `014a3ab0` | ActionNode::AddCustomDialogCommand | confirmed | Traces "Doing a CustomDialog". Requires key 0x1c (value-data list, at least two entries) and keys 0x17, 0x23, 0x24, 0x25 (dialog text and buttons). Builds a di… |
| `014a3e20` | ActionNode::AddRevealCommand | confirmed | Traces "Adding a Reveal command". Requires keys 0x16 (kNumActionsKey), 0x17 (kTargetTextKey), 0x18 (kRandomKey), 0x19 (kGetTargetKey) and kAltPlayAreaKey and k… |
| `0149f8a0` | ActionNode::CheckHand | likely | Traces "target == Hand". Walks the entry list: entries with key 7 are resolved with 01381460, 013815a0 and 0139d7c0 and passed to vtable[21](key, value, enviro… |
| `014a3650` | ActionNode::AddDiscardPileCommand | likely | Traces "Adding a DiscardPile command", then walks the entry list: entries with key 7 or 8 are resolved with 01381460, 013815a0, 01344ef0 and 0139d7b0 and passe… |

**ActionPlayNode** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0149e270` | ActionPlayNode::PlayCard | confirmed | Resolves the Card target with 01381460(0, PlayElement RTTI, Card RTTI, 0) and 00d8d382 (dynamic cast); returns 0 with trace text when there is no card. If the … |

### `minigame.tcg.command`

**CommandObject** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0142a730` | CommandObject::AssertFalseReturn0 | confirmed | Calls FUN_012f5a60('false', CommandObject.cpp, line 0x88) (an assert of false) and returns 0. |
| `0142af20` | CommandObject::Dtor | confirmed | Re-stores CommandObject::vftable at offsets 0 and 8, calls FUN_01344cd0 (no real argument), and when the byte at +0x80 is zero and the pointer at +0x70 is set,… |
| `0142b000` | CommandObject::GetName | confirmed | Returns the address of the literal 'CommandObject' (6 bytes, mov eax, imm32; ret). |
| `0142b010` | CommandObject::GetSharedDword | confirmed | Returns the dword global at 01cbe28c (6 bytes). |
| `0142b020` | CommandObject::SetSharedDword | confirmed | Stores its stack argument into the dword global at 01cbe28c (12 bytes, ret 4). |
| `0142b040` | CommandObject::AdjustThisScalarDeletingDtor | confirmed | Adjustor thunk (bytes 83 e9 08 / e9 08 00 00 00): subtracts 8 from ECX and jumps to 0142b050 (the scalar deleting destructor). |
| `0142b050` | CommandObject::ScalarDeletingDtor | confirmed | Calls FUN_0142af20 (the destructor body), then frees the object with _free when bit 0 of the second argument is set; returns the object pointer. |
| `0142b070` | CommandObject::CommandObject | confirmed | Default constructor: calls FUN_01343d20 (base constructor; the argument is a stack-cookie artifact), stores CommandObject::vftable at offsets 0 and 8, sets the… |
| `0142b150` | CommandObject::CommandObjectWithArg | likely | Same field setup as the default constructor 0142b070 (vtables, string at +0x48 assigned 'CommandObject', flags and counters), then calls FUN_01386b70 with its … |
| `0142b250` | CommandObject::Create | likely | Allocates 0x124 bytes with FUN_00d833a6; if the allocation succeeded it runs FUN_0142b070 (CommandObject default constructor) on the block and returns the bloc… |

**GameCommand_FatalError** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01429c70` | GameCommand_FatalError::Handle | confirmed | Asserts this+8 (mGame) is non-null (file line 0x4a), calls FUN_0141b020, then FUN_013487f0; if that returns nonzero it calls FUN_0136d810(this+0x24), otherwise… |
| `01429cc0` | GameCommand_FatalError::Clone | confirmed | Allocates 0x94 bytes with FUN_00d833a6; if the allocation is non-null it constructs the object through FUN_01429980(this[2]) and then calls this vtable slot 0x… |
| `01429d40` | GameCommand_FatalError::CopyFieldsTo | confirmed | Dynamic-casts the argument to GameCommand_FatalError (FUN_00d8d382 with the NetworkCommand and GameCommand_FatalError RTTI descriptors; asserts clone on null),… |
| `01429fe0` | GameCommand_FatalError::ScalarDeletingDtor | confirmed | Calls FUN_01429a10 (destructor body), then frees the object with _free when bit 0 of the second argument is set; returns the object pointer. |
| `0142a000` | GameCommand_FatalError::Dump | confirmed | Appends the text 'Fatal Error Game Command' with a newline through FUN_01241650, calls FUN_0141b240(param_1), reads the message string from the object returned… |
| `01429b70` | GameCommand_FatalError::CompressSerializedGame | likely | Takes the buffer and its length from FUN_013c2330 (begin pointer into a local, length into this+0x8c), copies the bytes into a new heap block, sets this+0x90 t… |

**GameCommand_IntroduceCard** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0142ebe0` | GameCommand_IntroduceCard::GetName | confirmed | Returns the address of the literal 'GameCommand_IntroduceCard' (6 bytes, mov eax, imm32; ret). |
| `0142ebf0` | GameCommand_IntroduceCard::GetSharedDword | confirmed | Returns the dword global at 01cbe294 (6 bytes). |
| `0142ec00` | GameCommand_IntroduceCard::SetSharedDword | confirmed | Stores its stack argument into the dword global at 01cbe294 (12 bytes, ret 4). |
| `0142ec20` | GameCommand_IntroduceCard::SetIntroducedCard | confirmed | Asserts the argument (a card object) is non-null (file GameCommandIntroduceCard.cpp, line 0x1f), stores FUN_01340370(card) at +0x24 (introduced card id) and FU… |
| `0142ed40` | GameCommand_IntroduceCard::CopyFieldsTo | confirmed | Dynamic-casts the argument to GameCommand_IntroduceCard (FUN_00d8d382 with NetworkCommand and GameCommand_IntroduceCard RTTI; asserts clone on null), calls FUN… |
| `0142eea0` | GameCommand_IntroduceCard::CtorDefault | confirmed | Calls FUN_0141b5d0 (base constructor; argument is a stack-cookie artifact), stores GameCommand_IntroduceCard::vftable, and zeroes the three dwords at +0x24, +0… |
| `0142ef00` | GameCommand_IntroduceCard::ScalarDeletingDtor | confirmed | Stores the class vtable, calls FUN_0141b310 (base destructor), and frees the object with _free when bit 0 of the second argument is set; returns the object poi… |
| `0142ef60` | GameCommand_IntroduceCard::CtorWithArg | confirmed | Calls FUN_0141b5d0, stores the class vtable, zeroes +0x28 and +0x2c, calls FUN_0141b3d0 with its argument (base constructor with argument), then zeroes +0x24, … |
| `0142efe0` | GameCommand_IntroduceCard::Clone | confirmed | Allocates 0x30 bytes with FUN_00d833a6; if the allocation is non-null it constructs the copy through FUN_0142ef60(this[2]) and then calls vtable slot 0x48 (slo… |
| `0142f0d0` | GameCommand_IntroduceCard::Dump | confirmed | Appends 'Introduce Card Game Command' through FUN_01241650, calls FUN_0141b240 on the argument, then formats 'Introduced Card ID: %d' and 'Introduced Instance … |
| `0142f050` | GameCommand_IntroduceCard::Create | likely | Allocates 0x30 bytes with FUN_00d833a6; if the allocation succeeded it runs FUN_0142eea0 (default constructor) on the block and returns the result; returns 0 o… |

**ValidActionFilter** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0142a0a0` | ValidActionFilter::ValidActionFilter | confirmed | Calls FUN_01343d20 (base constructor; the argument in the pseudocode is a stack-cookie artifact), stores ValidActionFilter::vftable at offset 0 and at offset 8… |
| `0142a130` | ValidActionFilter::GetName | confirmed | Returns the address of the literal 'ValidActionFilter' (bytes b8 0c 7a 8e 01 c3: mov eax, 018e7a0c; ret). |
| `0142a140` | ValidActionFilter::GetSharedDword | confirmed | Returns the dword global at 01cbe288 (bytes a1 88 e2 cb 01 c3: mov eax,[01cbe288]; ret). |
| `0142a150` | ValidActionFilter::SetSharedDword | confirmed | Stores its one stack argument into the dword global at 01cbe288 (bytes 8b 44 24 04 / a3 88 e2 cb 01 / c2 04 00: mov eax,[esp+4]; mov [01cbe288],eax; ret 4). |
| `0142a170` | ValidActionFilter::Dtor | confirmed | Re-stores ValidActionFilter::vftable at offsets 0 and 8, calls FUN_01344cd0 (no real argument) and then FUN_01344d10 (base-class teardown). Called by the scala… |
| `0142a5b0` | ValidActionFilter::AdjustThisScalarDeletingDtor | confirmed | Adjustor thunk (bytes 83 e9 08 / e9 a8 00 00 00): subtracts 8 from ECX and jumps to 0142a660 (the scalar deleting destructor). |
| `0142a660` | ValidActionFilter::ScalarDeletingDtor | confirmed | Calls FUN_0142a170 (the destructor body), then frees the object with _free when bit 0 of the second argument is set; returns the object pointer. |
| `0142a1f0` | ValidActionFilter::SetExpression | likely | Asserts the expression argument is non-null (file ValidActionFilter.cpp line 0x32), then stores it at +0x48, sets the byte at +0x4c to 1 and stores the third a… |
| `0142a5c0` | ValidActionFilter::Create | likely | Allocates 0x58 bytes with FUN_00d833a6; if the allocation succeeded it runs FUN_0142a0a0 (ValidActionFilter constructor) on the block and returns the block plu… |

### `minigame.tcg.data`

**Archetype** (24)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0137bdb0` | Archetype::FindIntListKey | confirmed | Virtual slot 59 of Archetype taking one int value on the stack. It reads property key 0x237 through this+4 (virtual slot 1, +4) and returns 0x237 when the valu… |
| `0137cd70` | Archetype::GetTcgTextProperty | confirmed | Virtual slot 27 of Archetype taking a property key and an out ValueData. Keys 1 or 0x22a65 format 'Tcg_title_%d', 5 or 0x22a6c 'Tcg_subtitle_%d', 0x276 or 0x22… |
| `01380480` | Archetype::GetTypeName | confirmed | Virtual slot 11 of Archetype with no arguments: returns the address of the literal 'Archetype' (018dd3f8) in EAX. |
| `013804c0` | Archetype::ScalarDeletingDtorThunk | confirmed | Adjustor thunk in the secondary vtable at 018dd2d0+4: subtracts 4 from ECX and jumps to Archetype::ScalarDeletingDtor (01380810), so the deleting destructor ru… |
| `01380760` | Archetype::Archetype | confirmed | Constructor (thiscall, this in ECX, no stack arguments, returns this in EAX). Stores PersistentBase::vftable (018d100c) then PersistentComponent::vftable (018d… |
| `01380810` | Archetype::ScalarDeletingDtor | confirmed | Scalar deleting destructor, thiscall with a flag byte (ret 4). Calls the destructor body 013803c0 on this, then calls _free(this) when bit 0 of the flag is set… |
| `01379550` | Archetype::ResolveSubobjectAt10 | likely | Virtual [14]: if the dword at this+0x18 is non-zero and virtual +0x50 of this returns a non-null object, returns that object's slot-14 result. Otherwise return… |
| `01379580` | Archetype::SetDwordAt10 | likely | Virtual [15]: when the dword at this+0x18 is non-zero it asserts 'false' through 012f5a60 (Archetype.cpp line 0x115), then stores the argument at this+0x10. |
| `01379600` | Archetype::GetLazyDwordAt1c | likely | Virtual [19]: when the dword at this+0x18 is non-zero and the value at +0x1c is still zero, calls 012b8f80 and then 012b8ff0 with (this+0x18, this+0xc) and sto… |
| `0137a030` | Archetype::CopyViaSerialization | likely | Virtual [65] taking an archetype pointer: asserts it is non-null (Archetype.cpp line 0x6e1), zeroes a 12-byte temporary through 013c2210, calls the argument's … |
| `0137b990` | Archetype::ApplyPropertyList | likely | Virtual slot 64 of Archetype. If this+0x10 is null it allocates a 0x44-byte block (00d833a6), builds the object through 014343d0 and stores it at this+0x10, th… |
| `0137c040` | Archetype::RemoveKeywordIntegerValue | likely | Virtual slot 61 of Archetype taking one int value on the stack. It reads property key 2 through this+4 (virtual slot 1), asserts keywordData.isIntegerListValue… |
| `0137c390` | Archetype::GetPronoun | likely | Virtual slot 52 of Archetype taking an int code and an output std::string. For codes 1, 2 and 3 it assigns one of three strings obtained from 013cb9b0, 013cb9d… |
| `0137c950` | Archetype::FormatCountNameLine | likely | Virtual slot 51 of Archetype with an out std::string and one int. It calls this virtual +0x5c to get a switch code; for codes other than 2, 3, 4 and 6 to 10 it… |
| `0137cc20` | Archetype::SymbolMarkupForIndex | likely | Virtual slot 35 of Archetype taking an int index. For indexes 1 to 9 it builds a std::string from one of nine rodata literals of the form '<symbol>' + one glyp… |
| `0137dbf0` | Archetype::SetIdAfterMaxInList | likely | Virtual slot 16 of Archetype with no arguments. It gathers the integers of the list held at this+0xc (through 012b8f80 and 012b8ae0) into a temporary vector, k… |
| `0137dd00` | Archetype::KeyedLookupOrForward | likely | Virtual slot 55 of Archetype taking three arguments. When 012aeb10 on this+8 is true it looks up the object for id minus 300000 through 012b8ff0 and forwards t… |
| `0137e340` | Archetype::SumIntValuesOverIdList | likely | Virtual slot 67 of Archetype taking a key and a second value. When arguments are zero it takes defaults through 012d08e0 and 013a3480 or 012d0780. It gets an i… |
| `0137e5b0` | Archetype::CollectIntValuesByIdList | likely | Virtual slot 68 of Archetype taking an out list and a key; zero defaults come from 012d46a0 and 012d08e0 or 013a3480. It gets an id list from this virtual +0xd… |
| `0137e7a0` | Archetype::BuildIdCountDisplayString | likely | Virtual slot 66 of Archetype with two optional arguments and an out ValueData. It gets an id list from this virtual +0xdc, tallies duplicate ids in std::map co… |
| `01380490` | Archetype::GetStaticDword | likely | Virtual slot 12 of Archetype: returns the global dword DAT_01cbdbcc. |
| `013804a0` | Archetype::SetStaticDword | likely | Virtual slot 13 of Archetype taking one stack dword: stores it into the global DAT_01cbdbcc and returns with ret 4. |
| `01380830` | Archetype::Factory | likely | Factory: allocates 0x34 (52) bytes with operator new 00d833a6; if the pointer is null it returns 0, otherwise it calls the Archetype constructor 01380760 on th… |
| `01380890` | Archetype::RegisterFactory | likely | Registers factory 01380830 under classID 3 by calling 013ce130(3, 01380830) and then 013ce230(3, 01380830), then returns. |

**Deck** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013ce5b0` | Deck::~Deck | confirmed | Destructor body: resets vtable to Deck, PersistentComponent and PersistentBase; frees the vector buffer at +0x7c when non-null; for each of the four std::strin… |
| `013ce720` | Deck::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the body 013ce5b0, then calls _free on this when bit 0 of the second argument is set; returns this. |
| `013ce870` | Deck::Deck | confirmed | Constructor: stores PersistentBase, PersistentComponent and Deck vtables; assigns empty strings to the four string members at +0x04, +0x20, +0x3c, +0x58; reset… |
| `013cead0` | Deck::Clone | confirmed | Virtual slot 29 of Deck: allocates 0x94 bytes with 00d833a6, constructs a Deck there (013ce870), copies the strings at this+0x04 and this+0x3c into it, calls s… |
| `013cf400` | Deck::LoadTextFile | confirmed | Virtual slot 28 of Deck (stdcall: file name and logger arguments): opens the file for reading (exit with Couldn't open text file on failure); reads each line t… |
| `013ce4f0` | Deck::Serialize | likely | Virtual slot 10 of Deck: calls slot 1 when the argument is 0, otherwise slot 2 with the argument; then SerialAppendString 013c2860 for the string objects at +0… |
| `013ce7c0` | Deck::ReadFileLine | likely | Virtual slot 26 of Deck (also in CWDeck slot 26): reads characters from a FILE with 0110b4b4 until newline or end of file, up to 1000 characters into a stack b… |
| `013ceb90` | Deck::Create | likely | Allocates 0x94 bytes with 00d833a6; when the allocation is non-null it constructs a Deck with 013ce870 and returns it, otherwise returns 0. |
| `013cef20` | Deck::Deserialize | likely | Virtual slot 9 of Deck: reads a header through slot 4, then SerialReadString 013c28d0 into the strings at +0x04, +0x20, +0x3c, +0x58, a byte at +0x74 via 013c2… |

**Engine::CardRefBase** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01380ba0` | Engine::CardRefBase::ScalarDeletingDtor | confirmed | Scalar deleting destructor, thiscall with a flag byte (ret 4). Stores Engine::CardRefBase::vftable at +0, calls base destructor 01312690 on this, then calls _f… |

**Engine::EngineUpdateDataVersionDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01250230` | Engine::EngineUpdateDataVersionDisplayAction::CacheDataVers… | confirmed | Slot [2] of Engine::EngineUpdateDataVersionDisplayAction::vftable, 1362 bytes. Gets the thread-owned object (FUN_0124f2e0); reads ValueData entry 0 as the inte… |

**InstanceID** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013d3300` | InstanceID::InstanceID | confirmed | Constructor: stores PersistentBase, PersistentComponent and InstanceID vtables, then zeroes the three dwords at +0x04, +0x08 and +0x0c; returns nothing. |
| `013d3390` | InstanceID::CommandName | confirmed | Virtual slot 11: returns the address of the literal InstanceID. |
| `013d33d0` | InstanceID::InstanceID_TwoFields | confirmed | Constructor with two arguments: stores PersistentBase, PersistentComponent and InstanceID vtables, zeroes the dword at +0x08, stores the second argument at +0x… |
| `013d3450` | InstanceID::InstanceID_ThreeFields | confirmed | Constructor with three arguments: stores PersistentBase, PersistentComponent and InstanceID vtables, stores the arguments at +0x04, +0x08 and +0x0c. |
| `013d34d0` | InstanceID::~InstanceID | confirmed | Destructor body: stores InstanceID, PersistentComponent and PersistentBase vtables in that order; no members are freed. |
| `013d3720` | InstanceID::ScalarDeletingDtor | confirmed | Scalar deleting destructor with the body inlined: stores InstanceID, PersistentComponent and PersistentBase vtables, then calls _free on this when bit 0 of the… |
| `013d3790` | InstanceID::InstanceID_Copy | confirmed | Copy constructor: stores PersistentBase, PersistentComponent and InstanceID vtables, then copies the dwords at +0x04, +0x08 and +0x0c from the argument. |
| `013d35b0` | InstanceID::Deserialize | likely | Virtual slot 9: reads a header through slot 4, then three integers into +0x04, +0x08 and +0x0c with 013c24d0; calls slot 5 (+0x14); returns 0 on any failed rea… |
| `013d3610` | InstanceID::Serialize | likely | Virtual slot 10: calls slot 2 with its argument, writes trace labels through 013c26e0 (no-op), appends the dwords at +0x04, +0x08 and +0x0c with SerialAppendIn… |
| `013d36a0` | InstanceID::Create | likely | Allocates 0x10 bytes with 00d833a6; when non-null constructs an InstanceID with 013d3300 and returns it, otherwise returns 0. |

**(no class)** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0131cee0` | CardAttributeNameForId | confirmed | Cdecl-style, one int arg (attribute id). Returns a C string literal from a range-partitioned switch: ids 0x1 to 0x12c and 0x12c (DisplayCardImage), the 0x1-0x2… |
| `0131f270` | WriteUnitsCsvFile | confirmed | Thiscall (ECX = object), arg p2 = output path. Opens p2 for writing with _fopen, writes the 'UNITS' header line, then walks the tree at this+0x8c (header at th… |
| `01517320` | StaticNameTableInit | confirmed | Sets the once-guard DAT_01cbeb44 to 1, then fills nine global key-to-string maps (DAT_01cbeb48, 01cbeb54, 01cbeb60, 01cbeb6c, 01cbeb78, 01cbeb84, 01cbeb90, 01c… |
| `013794e0` | ArchetypeSetDwordAt0c | likely | Thiscall setter. When the argument is -1 it calls the assert helper 012f5a60 with the text 'v != -1' and Archetype.cpp line 0x9d; then it stores the argument a… |
| `0137da80` | ReadCountPrefixedKeyTree | likely | Reads a count through 013c2690 (SerialStartReadContainer) and then that many entries: each entry is read through 014001c0 into a key and a value, the value is … |
| `015192b0` | StringMapLookupCopy_01cbeb48 | likely | Runs 01517320 once when DAT_01cbeb44 is zero, takes the entry for the stack key from the map at DAT_01cbeb48 through 012a49c0, and assigns a copy of that std::… |
| `01519340` | StringMapLookupCopy_01cbeb78 | likely | Runs 01517320 once when DAT_01cbeb44 is zero, takes the entry for the stack key from the map at DAT_01cbeb78 through 012a49c0, and assigns a copy of that std::… |
| `015193d0` | StringMapLookupCopy_01cbeb90 | likely | Runs 01517320 once when DAT_01cbeb44 is zero, takes the entry for the stack key from the map at DAT_01cbeb90 through 012a49c0, and assigns a copy of that std::… |
| `01519460` | StringMapLookupCopy_01cbeb54 | likely | Runs 01517320 once when DAT_01cbeb44 is zero, takes the entry for the stack key from the map at DAT_01cbeb54 through 012a49c0, and assigns a copy of that std::… |
| `015194f0` | StringMapLookupCopy_01cbeba8 | likely | Runs 01517320 once when DAT_01cbeb44 is zero, takes the entry for the stack key from the map at DAT_01cbeba8 through 012a49c0, and assigns a copy of that std::… |
| `01519580` | StringMapLookupCopy_01cbeb9c | likely | Runs 01517320 once when DAT_01cbeb44 is zero, takes the entry for the stack key from the map at DAT_01cbeb9c through 012a49c0, and assigns a copy of that std::… |
| `01519610` | StringMapLookupCopy_01cbeb84 | likely | Runs 01517320 once when DAT_01cbeb44 is zero, takes the entry for the stack key from the map at DAT_01cbeb84 through 012a49c0, and assigns a copy of that std::… |
| `015196a0` | StringMapLookupCopy_01cbeb60 | likely | Runs 01517320 once when DAT_01cbeb44 is zero, takes the entry for the stack key from the map at DAT_01cbeb60 through 012a49c0, and assigns a copy of that std::… |
| `01519730` | StringMapLookupCopy_01cbeb6c | likely | Runs 01517320 once when DAT_01cbeb44 is zero, takes the entry for the stack key from the map at DAT_01cbeb6c through 012a49c0, and assigns a copy of that std::… |

### `minigame.tcg.game`

**CTP::Game** (19)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01347d30` | CTP::Game::GetMyPlayerID | confirmed | Fastcall. Asserts this+0x13c is non-null (Game.cpp line 0x10a5, text mMyPlayerID) and returns the dword at this+0x13c. |
| `01347d60` | CTP::Game::SetMyPlayerID | confirmed | Thiscall with one stack argument (ret 4). Asserts the argument is non-zero (Game.cpp line 0x10ab, text playerid) and stores it at this+0x13c. |
| `013488e0` | CTP::Game::GetNodeTarget | confirmed | Virtual slot [128] of CTP::Game::vftable. Traces 'Game::getNodeTarget %d' through 013846b0 and maps the node target id argument (switch on values 1 to 0x420; o… |
| `01347d00` | CTP::Game::ForwardCurrentTurnSlot22 | likely | Fastcall. Asserts mCurrentTurn (this+0x68) non-null (Game.cpp line 0xd2b, text mCurrentTurn), then tail-jumps to the mCurrentTurn vtable slot +0x58 (slot 22) w… |
| `01348010` | CTP::Game::ForwardCurrentStateSlot66 | likely | Virtual slot [66] of CTP::Game::vftable (fastcall, ECX=this). Asserts mCurrentTurn (this+0x68) non-null (Game.cpp line 0x12a9) and mCurrentTurn->getCurrentStat… |
| `01348070` | CTP::Game::ForwardCurrentStateSlot67 | likely | Virtual slot [67] of CWGame::vftable and CWMainController::vftable (shared CTP::Game implementation). Asserts mCurrentTurn (this+0x68) and mCurrentTurn->getCur… |
| `013480d0` | CTP::Game::ForwardCurrentStateSlot68 | likely | Virtual slot [68] of CWGame and CWMainController (shared CTP::Game implementation). Asserts mCurrentTurn and its getCurrentStateMachine() (Game.cpp line 0x12b8… |
| `01348130` | CTP::Game::ForwardCurrentStateSlot69 | likely | Virtual slot [69] of CWGame and CWMainController. Asserts mCurrentTurn and its getCurrentStateMachine() (Game.cpp line 0x12bf), then tail-jumps to 013efdb0 wit… |
| `01348190` | CTP::Game::ForwardCurrentStateSlot70 | likely | Virtual slot [70] of CWGame and CWMainController. Asserts mCurrentTurn and its getCurrentStateMachine() (Game.cpp line 0x12c6), then tail-jumps to 013efde0 wit… |
| `013481f0` | CTP::Game::ForwardCurrentStateSlot71 | likely | Virtual slot [71] of CWGame and CWMainController. Asserts mCurrentTurn and its getCurrentStateMachine() (Game.cpp line 0x12cd), then tail-jumps to 013efe10 wit… |
| `01348250` | CTP::Game::ForwardCurrentStateSlot72 | likely | Virtual slot [72] of CWGame and CWMainController. Asserts mCurrentTurn and its getCurrentStateMachine() (Game.cpp), then calls 013efe70 with ECX set to the sta… |
| `013482b0` | CTP::Game::ForwardCurrentStateSlot74 | likely | Virtual slot [74] of CWGame and CWMainController. Asserts mCurrentTurn and its getCurrentStateMachine() (Game.cpp line 0x12db), then tail-jumps to 013efe40 wit… |
| `01348380` | CTP::Game::ForwardCurrentStateSlot76 | likely | Virtual slot [76] of CWGame and CWMainController. Asserts mCurrentTurn and its getCurrentStateMachine() (Game.cpp line 0x12ee), then tail-jumps to 013eff00 wit… |
| `013483e0` | CTP::Game::ForwardCurrentStateSlot77 | likely | Virtual slot [77] of CWGame and CWMainController. Asserts mCurrentTurn and its getCurrentStateMachine() (Game.cpp line 0x12f5), then tail-jumps to 013eff30 wit… |
| `01348440` | CTP::Game::ForwardCurrentStateSlot78 | likely | Virtual slot [78] of CWGame and CWMainController. Asserts mCurrentTurn and its getCurrentStateMachine() (Game.cpp line 0x12fc), then tail-jumps to 013eff60 wit… |
| `013484a0` | CTP::Game::ForwardCurrentStateUnslotted | likely | Asserts mCurrentTurn and its getCurrentStateMachine() (Game.cpp line 0x1304), then tail-jumps to 013eff90 with ECX set to the state machine. Same shape as the … |
| `01348500` | CTP::Game::ForwardCurrentStateSlot79 | likely | Virtual slot [79] of CWGame and CWMainController. Asserts mCurrentTurn and its getCurrentStateMachine() (Game.cpp line 0x130b), then tail-jumps to 013effc0 wit… |
| `01348ca0` | CTP::Game::VSlot115CreatePlayer | likely | Virtual slot [115] of CTP::Game::vftable (shared with CWGame and CWMainController). Allocates 0x4c bytes with operator new (00d833a6); when non-null constructs… |
| `01357a00` | TwoMapIntKeyLookupValue | likely | Thiscall, one stack arg (int key). Looks the key up with 013550c0 in the container at this+0xa4 (sentinel word at +0xa8, end at +0xac); if found returns node v… |

### `minigame.tcg.playelement`

**PlayElement** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01343e30` | PlayElement::GetTypeName | confirmed | Returns the address of the literal PlayElement (mov eax,0x18dabf8; ret). |
| `01343e40` | PlayElement::GetStaticDword | confirmed | Returns the global dword DAT_01cbdbac (mov eax,[0x1cbdbac]; ret). Slot [12] of the secondary vtable. |
| `01343e50` | PlayElement::SetStaticDword | confirmed | Stores its single stack argument into the global dword DAT_01cbdbac (mov eax,[esp+4]; mov [0x1cbdbac],eax; ret 4). Slot [13] of the secondary vtable. |
| `01343e70` | PlayElement::ScalarDeletingDtorThunk | confirmed | Adjustor thunk at slot [0] of the secondary vtable (object+8): sub ecx,8 then jmp 01344df0 (the scalar deleting destructor of the primary subobject). It does n… |
| `013443c0` | PlayElement::ChangeParent | confirmed | Thiscall with a new parent (param_2) and a containment type (param_3). Traces the change through 013508b0 with the parent id at parent+0x34 and the containment… |
| `01344d10` | PlayElement::DtorBody | confirmed | Destructor body (fastcall, ECX=this). Stores PlayElement::vftable at this+0 and this+8. If this+0x30 is set calls 01357960 on this+0x34. Calls the primary vtab… |
| `01344df0` | PlayElement::ScalarDeletingDtor | confirmed | Thiscall with a flag argument. Calls the destructor body 01344d10 on this, then when bit 0 of the flag is set calls _free on this (through the 00d82463 thunk).… |

### `minigame.tcg.player`

**Player** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01345d50` | Player::DtorBody | confirmed | Destructor body (fastcall, ECX=this). Stores Player::vftable, then when the pointer at this+4 is set calls 01344cd0 on it, calls its scalar deleting destructor… |
| `01345e30` | Player::GetTypeName | confirmed | Returns the address of the literal Player at 018ca06c (mov eax,0x18ca06c; ret). Slot [11] of the Player vtable. |
| `01345e40` | Player::GetStaticDword | confirmed | Returns the global dword DAT_01cbdbb0 (mov eax,[0x1cbdbb0]; ret). Slot [12] of the Player vtable. |
| `01345e50` | Player::SetStaticDword | confirmed | Stores its stack argument into the global dword DAT_01cbdbb0 (ret 4). Slot [13] of the Player vtable. |
| `01345ed0` | Player::ScalarDeletingDtor | confirmed | Thiscall with a flag argument. Calls Player::DtorBody (01345d50) on this, then when bit 0 of the flag is set calls _free on this (through the 00d82463 thunk). … |
| `01345ef0` | Player::Player | confirmed | Constructor (ECX=this). Stores PersistentBase, PersistentComponent and Player vtables at offset 0, sets the string fields at this+0xc (size 0 at +0x20, capacit… |
| `013463d0` | Player::TraceAccountPrefs | confirmed | Fastcall with this (ECX). Asserts that mGame (this+8) and mAccountID (this+0x38) are non-null (Player.cpp lines 0xd9, 0xda). Traces the line AccountPref header… |
| `01345fa0` | Player::VSlot14CreateHolderCard | likely | Virtual slot [14] of Player::vftable. Formats the string Player%d HolderCard with _sprintf using the id at this+0x34, allocates an object with operator new and… |
| `01346b60` | Player::VSlot30DrawCards | likely | Virtual slot [30] of Player::vftable (and CWPlayer). For up to param_2 iterations: takes the next entry from a collection through 0139d7a0/0139f5d0, calls the … |
| `01347070` | Player::VSlot26PlayCard | likely | Virtual slot [26] of Player::vftable (and CWPlayer). Looks up the card id argument with 01394b10 (returns 0 when absent). Calls the object's virtual at +0x100 … |

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0135a490` | PlayerNameOrDefault | confirmed | Stdcall-style (ret 8) with two stack args (out std::string, player id). Assigns the out string from 013a6540 when 012de760 reports a non-zero result, then if t… |
| `0135a5d0` | PlayerNameByKey | confirmed | Thiscall (ECX = object) with two stack args (out std::string, key), ret 8. Searches the object by key through 01350660; when nothing matches it assigns 'Unknow… |
| `0135a690` | PlayerPossessiveOrDefault | likely | Thiscall (ECX = object) with two stack args (out std::string, key), ret 8. When 01350660 finds the key it reads the field at +0x38 through 01344e60, calls 012d… |
| `0135a7e0` | PlayerObjectPronounOrDefault | likely | Thiscall (ECX = object) with two stack args (out std::string, key), ret 8. Same shape as 0135a690: finds the key through 01350660, and on success copies the pr… |

### `minigame.tcg.property`

**ArchetypeDB** (17)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012b82d0` | ArchetypeDB_OpenStore | confirmed | Thiscall. Asserts the id is not -1 (ArchetypeDB.cpp line 0x346). Looks up the id in the map whose header is at this+0x18 (head at this+0x1c) through 0129adc0. … |
| `012b87e0` | ArchetypeDB_DecodeArchetype | confirmed | Asserts the id is not -1 (ArchetypeDB.cpp line 0x1b7). Sizes a byte buffer to param_3+1 through 013c2370, uncompresses the stored blob through 01539dc0, and as… |
| `012b8ff0` | ArchetypeDB_GetArchetype | confirmed | Thiscall-style accessor with (archetype id, store id) arguments. Asserts the store id is not -1 (ArchetypeDB.cpp line 0x15a), opens the store (012b82d0) and lo… |
| `012b9760` | ArchetypeDB_WriteStore | confirmed | Asserts the id is not -1 (ArchetypeDB.cpp line 0x279). Serialises the id's records into a byte buffer: for each index entry it appends the id, the byte length … |
| `012b7810` | ArchetypeDB_EnsureIdEntry | likely | Thiscall with two stack arguments (ret 8). Asserts the id argument is not -1 (ArchetypeDB.cpp line 0x309). Find-or-insert of the id in the int-keyed tree at th… |
| `012b7fe0` | ArchetypeDB_RegisterArchetype | likely | Asserts the id is not -1 (ArchetypeDB.cpp line 0x1db) and the archetype pointer is non-null (line 0x1dd), calls 013794e0 with the id and 013795c0, calls virtua… |
| `012b80e0` | ArchetypeDB_RemoveArchetype | likely | Asserts the id is not -1 (ArchetypeDB.cpp line 0x1ea), looks up the id in the map header at this+0xc, and when the per-id entry exists (012b7dc0) finds the sto… |
| `012b8530` | ArchetypeDB_LoadIndex | likely | Thiscall. Asserts the id is not -1 (ArchetypeDB.cpp line 0x104). Looks up the id in the map whose header is at this+0x30 (head at this+0x34) through 0129ad50 a… |
| `012b89d0` | ArchetypeDB_AppendIndexFirstIds | likely | Thiscall. Asserts the id is not -1 (ArchetypeDB.cpp line 0x2c7), opens the store through 012b82d0, then walks the per-id entry from 012b7c80 and appends the fi… |
| `012b8ae0` | ArchetypeDB_CollectIdsDedup | likely | Thiscall. Asserts the id is not -1 (ArchetypeDB.cpp line 0x2db) and opens the store through 012b82d0. First pass: appends the first dword of each element of th… |
| `012b8df0` | ArchetypeDB_EntryContainsValue | likely | Asserts the id is not -1 (ArchetypeDB.cpp line 0x310), opens the store through 012b82d0, looks up the id through 01247a90 and scans the entry's int list from b… |
| `012b8f50` | ArchetypeDB_SetActiveArchetype | likely | Thiscall with one argument. When the argument is -1 it stores 0x9f3 (2547) at this+0x3c, otherwise it stores the argument there; then calls 012b8530 (the index… |
| `012b8f80` | ArchetypeDB_Instance | likely | Lazy singleton getter. When global DAT_01cbd688 is null it allocates 0x40 bytes with operator new (00d833a6); on success it stores the result of 012b8eb0 (the … |
| `012b9330` | ArchetypeDB_LoadAllArchetypes | likely | Asserts the id is not -1 (ArchetypeDB.cpp line 0x208), opens the store through 012b82d0, collects the id list of the index through 012b89d0 into a temporary in… |
| `012b9450` | ArchetypeDB_CollectArchetypeList | likely | Asserts the id is not -1 (ArchetypeDB.cpp line 0x218), opens the store through 012b82d0, collects the index ids through 012b89d0, passes each non-null GetArche… |
| `012b95c0` | ArchetypeDB_CollectArchetypesThen | likely | Thiscall with the receiver in ECX (kept in EDI) and two stack arguments. Asserts the id is not -1 (ArchetypeDB.cpp line 0x22a), opens the store through 012b82d… |
| `012b9be0` | ArchetypeDB_ForwardToArchetypeSlot_E8 | likely | Asserts the store id is not -1 (ArchetypeDB.cpp line 0x2f6), looks up the archetype through 012b8ff0 with the next two arguments, and when the archetype is non… |

**HasProperties** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01343dd0` | HasProperties::PropertySetForward | likely | Thiscall with two stack arguments (ret 8). If this+4 (mPropertySet) is null it calls 012f5a60 with the string mPropertySet and the HasProperties.h path (line 0… |

**(no class)** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012bccd0` | ActionDB_OpenArchetypeStore | confirmed | Asserts the id is not -1 (ActionDB.cpp line 0x284). If the archetype id is absent from the map at this+0x10 it allocates 0x58 bytes, constructs the store objec… |
| `012bcf20` | ActionDB_LoadActionIndex | confirmed | Asserts the id is not -1 (ActionDB.cpp line 0xe3), opens the archetype store through 012bccd0, and reads the actionIndex record through vtable slot +0x14 of th… |
| `012bd180` | ActionDB_RegisterAction | confirmed | Asserts the id is not -1 (ActionDB.cpp line 0x183) and the action pointer param_1 is non-null (line 0x185, assertion text action); opens the archetype through … |
| `012bd4a0` | ActionDB_LoadActionFromStore | confirmed | Asserts the id is not -1 (ActionDB.cpp line 0x137), opens the archetype through 012bccd0 and 012bcf20, reads the actionStorage blob via vtable slot +0x24 of th… |
| `012bd810` | ActionDB_SaveActionStorage | confirmed | Writes an archetype's data into a serial buffer: the archetype id and a __time64 word are appended twice each through 013c23f0 as a header; then for every inde… |
| `012bd2b0` | ActionDB_CollectActionIds | likely | Asserts the id is not -1 (ActionDB.cpp line 0x1fd), opens the archetype through 012bccd0, walks the tree returned by 0129f1b0 and appends the int at each node+… |
| `012bd3d0` | DtorBody_ClearsTwoTrees | likely | Destructor body: calls 012bcb40 (clears the owned-value tree at +0x14 and its nested data), calls 012bcc60 (clears the tree at +4 in an SEH frame), then tears … |
| `012bd450` | ArchetypeIdSetAndLoadIndex | likely | Stores param_2 in the first dword of this, or 0x9f3 when param_2 is -1, then tail-jumps to 012bcf20 with the same id so the archetype action index is loaded. |
| `012bdd30` | ActionDB_ClearGlobalAndFree | likely | Sets DAT_01cbd68c to 0; when the object in ECX is non-null it runs 012bd3d0 on it and frees it with _free. |
| `012bdd60` | ActionDB_Ctor | likely | Initialises member blocks through 0129d6c0, 0129d730, 012a0600 and 012bceb0 (archetype header init), stores 0x9f3 as the first dword of this and calls 012bcf20… |
| `012bddf0` | GlobalObj01cbd68c_GetOrCreate | likely | If DAT_01cbd68c is zero it allocates 0x34 bytes, and when the allocation succeeds stores the object built by 012bdd60 into DAT_01cbd68c; otherwise it leaves th… |
| `012bde60` | GlobalObj01cbd68c_Destroy | likely | When DAT_01cbd68c is non-null, runs 012bd3d0 on the object and frees it, then sets DAT_01cbd68c to 0. |

### `minigame.tcg.query`

**LastActionQueryNode** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01498290` | LastActionQueryNode::LastActionQueryNode | confirmed | Constructor under a stack-cookie frame. Runs the QueryNode constructor 01494290, stores LastActionQueryNode::vftable into *this and returns this. |
| `01498310` | LastActionQueryNode::GetTypeName | confirmed | Returns the literal string 'LastActionQueryNode' (vtable slot [11]). |
| `01498350` | LastActionQueryNode::~LastActionQueryNode | confirmed | Destructor body. Stores LastActionQueryNode::vftable into *this and calls the QueryNode destructor body 01493fd0. |
| `01498540` | LastActionQueryNode::ScalarDeletingDtor | confirmed | Scalar deleting destructor, virtual slot [0] of LastActionQueryNode. Stores the vtable, calls the QueryNode destructor body 01493fd0, and frees this with _free… |
| `014984c0` | LastActionQueryNode::Factory | likely | Allocates 0x24 bytes with operator new (00d833a6), runs LastActionQueryNode::LastActionQueryNode (01498290) on the block and returns it; returns 0 when the all… |
| `01498520` | LastActionQueryNode::RegisterFactory | likely | Registration initialiser. Calls 013ce130 with classID 0x2f and factory 014984c0, then 013ce230 with the same pair. |

**LessEqualExpression** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01497ee0` | LessEqualExpression::LessEqualExpression | confirmed | Constructor under a stack-cookie frame. Calls the base constructor 0149adc0, stores LessEqualExpression::vftable into *this and returns this. |
| `01497f60` | LessEqualExpression::GetTypeName | confirmed | Returns the literal string 'LessEqualExpression' (vtable slot [11]). |
| `01497fa0` | LessEqualExpression::~LessEqualExpression | confirmed | Destructor body. Stores LessEqualExpression::vftable into *this and calls the base destructor 0149ae80. |
| `01497ff0` | LessEqualExpression::Evaluate | confirmed | Virtual slot [15] of LessEqualExpression. Card-set evaluation is not implemented: calls 012f5a60 with 'false', the LessEqualExpression.cpp path and line 0x12, … |
| `01498010` | LessEqualExpression::EvaluateValue | confirmed | Virtual slot [14] of LessEqualExpression. Traces '(less equal', evaluates operands this+0x1c and this+0x20 through vtable +0x38 (returns 0 with ') => false' if… |
| `01498230` | LessEqualExpression::ScalarDeletingDtor | confirmed | Scalar deleting destructor, virtual slot [0] of LessEqualExpression. Stores the vtable, calls the base destructor 0149ae80, and frees this with _free when bit … |
| `014981b0` | LessEqualExpression::Factory | likely | Allocates 0x24 bytes with operator new (00d833a6), runs LessEqualExpression::LessEqualExpression (01497ee0) on the block and returns it; returns 0 when the all… |
| `01498210` | LessEqualExpression::RegisterFactory | likely | Registration initialiser. Calls 013ce130 with classID 0x21 and factory 014981b0, then 013ce230 with the same pair. |

**LessThanExpression** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01497850` | LessThanExpression::LessThanExpression | confirmed | Constructor under a stack-cookie frame. Calls the base constructor 0149adc0, stores LessThanExpression::vftable into *this and returns this. |
| `014978d0` | LessThanExpression::GetTypeName | confirmed | Returns the literal string 'LessThanExpression' (vtable slot [11]). |
| `01497910` | LessThanExpression::~LessThanExpression | confirmed | Destructor body. Stores LessThanExpression::vftable into *this and calls the base destructor 0149ae80. |
| `01497960` | LessThanExpression::EvaluateValue | confirmed | Virtual slot [14] of LessThanExpression. Traces '(less than', evaluates operand this+0x1c through vtable +0x38 and then operand this+0x20 with integer type che… |
| `01497b80` | LessThanExpression::ScalarDeletingDtor | confirmed | Scalar deleting destructor, virtual slot [0] of LessThanExpression. Stores the vtable, calls the base destructor 0149ae80, and frees this with _free when bit 0… |
| `01497b00` | LessThanExpression::Factory | likely | Allocates 0x24 bytes with operator new (00d833a6), runs LessThanExpression::LessThanExpression (01497850) on the block and returns it; returns 0 when the alloc… |
| `01497b60` | LessThanExpression::RegisterFactory | likely | Registration initialiser. Calls 013ce130 with classID 0x1d and factory 01497b00, then 013ce230 with the same pair. |

**MaxExpression** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014974d0` | MaxExpression::MaxExpression | confirmed | Constructor under a stack-cookie frame. Calls the base constructor 0149adc0, stores MaxExpression::vftable into *this and returns this. |
| `01497550` | MaxExpression::GetTypeName | confirmed | Returns the literal string 'MaxExpression' (vtable slot [11]). |
| `01497590` | MaxExpression::~MaxExpression | confirmed | Destructor body. Stores MaxExpression::vftable into *this and calls the base destructor 0149ae80. |
| `014975e0` | MaxExpression::Evaluate | confirmed | Virtual slot [15] of MaxExpression. Card-set evaluation is not implemented: calls 012f5a60 with 'false', the MaxExpression.cpp path and line 0x12, then returns… |
| `014977f0` | MaxExpression::ScalarDeletingDtor | confirmed | Scalar deleting destructor, virtual slot [0] of MaxExpression. Stores the vtable, calls the base destructor 0149ae80, and frees this with _free when bit 0 of t… |
| `01497770` | MaxExpression::Factory | likely | Allocates 0x24 bytes with operator new (00d833a6), runs MaxExpression::MaxExpression (014974d0) on the block and returns it; returns 0 when the allocation fail… |
| `014977d0` | MaxExpression::RegisterFactory | likely | Registration initialiser. Calls 013ce130 with classID 0x91 and factory 01497770, then 013ce230 with the same pair. |

**MinExpression** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01497150` | MinExpression::MinExpression | confirmed | Constructor under a stack-cookie frame. Calls the base constructor 0149adc0, stores MinExpression::vftable into *this and returns this. |
| `014971d0` | MinExpression::GetTypeName | confirmed | Returns the literal string 'MinExpression' (vtable slot [11]). |
| `01497210` | MinExpression::~MinExpression | confirmed | Destructor body. Stores MinExpression::vftable into *this and calls the base destructor 0149ae80. |
| `01497260` | MinExpression::Evaluate | confirmed | Virtual slot [15] of MinExpression. Card-set evaluation is not implemented: calls 012f5a60 with 'false', the MinExpression.cpp path and line 0x12, then returns… |
| `01497470` | MinExpression::ScalarDeletingDtor | confirmed | Scalar deleting destructor, virtual slot [0] of MinExpression. Stores the vtable, calls the base destructor 0149ae80, and frees this with _free when bit 0 of t… |
| `014973f0` | MinExpression::Factory | likely | Allocates 0x24 bytes with operator new (00d833a6), runs MinExpression::MinExpression (01497150) on the block and returns it; returns 0 when the allocation fail… |
| `01497450` | MinExpression::RegisterFactory | likely | Registration initialiser. Calls 013ce130 with classID 0x7f and factory 014973f0, then 013ce230 with the same pair. |

**ModulusExpression** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01496de0` | ModulusExpression::ModulusExpression | confirmed | Constructor under an SEH frame. Calls the base constructor 0149adc0, stores ModulusExpression::vftable into *this and returns this. |
| `01496e60` | ModulusExpression::GetTypeName | confirmed | Returns the literal string 'ModulusExpression' (vtable slot [11]). |
| `01496ea0` | ModulusExpression::~ModulusExpression | confirmed | Destructor body. Stores ModulusExpression::vftable into *this and calls the base destructor 0149ae80. |
| `01496ef0` | ModulusExpression::Evaluate | confirmed | Virtual slot [15] of ModulusExpression. Card-set evaluation is not implemented: calls 012f5a60 with 'false', the ModulusExpression.cpp path and line 0x12, then… |
| `01496f10` | ModulusExpression::EvaluateValue | confirmed | Virtual slot [14] of ModulusExpression. Traces '(Modulus', evaluates operands at this+0x1c and this+0x20 through vtable +0x38, requires integer type tags 2 on … |
| `014970f0` | ModulusExpression::ScalarDeletingDtor | confirmed | Scalar deleting destructor, virtual slot [0] of ModulusExpression. Stores the vtable, calls the base destructor 0149ae80, and frees this with _free when bit 0 … |
| `01497070` | ModulusExpression::Factory | likely | Allocates 0x24 bytes with operator new (00d833a6), runs ModulusExpression::ModulusExpression (01496de0) on the block and returns it; returns 0 when the allocat… |
| `014970d0` | ModulusExpression::RegisterFactory | likely | Registration initialiser. Calls 013ce130 with classID 0x17d and factory 01497070, then 013ce230 with the same pair. |

**MoveActionNode** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01496710` | MoveActionNode::MoveActionNode | confirmed | Constructor under an SEH frame. Runs the base constructor 0149ab30, stores MoveActionNode::vftable into *this and returns this. |
| `014967a0` | MoveActionNode::GetTypeName | confirmed | Returns the literal string 'MoveActionNode' (vtable slot [11]). |
| `014967e0` | MoveActionNode::~MoveActionNode | confirmed | Destructor body. Stores MoveActionNode::vftable into *this and calls the base destructor 0149a860. |
| `01496960` | MoveActionNode::EvaluateValue | confirmed | Virtual slot [14] of MoveActionNode. Traces '( MoveActionNode' and 'Move From', selects the source card through slot [20] with this+0x14, casts it PlayElement-… |
| `01496b50` | MoveActionNode::Evaluate | confirmed | Virtual slot [15] of MoveActionNode. Traces '(gvt:MoveActionNode' and ') => true', then returns 1 without touching the cards. |
| `01496d80` | MoveActionNode::ScalarDeletingDtor | confirmed | Scalar deleting destructor, virtual slot [0] of MoveActionNode. Stores the vtable, calls the base destructor 0149a860, and frees this with _free when bit 0 of … |
| `01496880` | MoveActionNode::SelectSource | likely | Virtual slot [20] of MoveActionNode. Switches on its selector argument: 1 calls 01381460, 2 calls 01381d40, 0x11 calls 013813e0, 0x2a calls 01383790, 0x30 call… |
| `01496d00` | MoveActionNode::Factory | likely | Allocates 0x24 bytes with operator new (00d833a6), runs MoveActionNode::MoveActionNode (01496710) on the block and returns it; returns 0 when the allocation fa… |
| `01496d60` | MoveActionNode::RegisterFactory | likely | Registration initialiser. Calls 013ce130 with classID 0x73 and factory 01496d00, then 013ce230 with the same pair. |

**MultiplyExpression** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014963a0` | MultiplyExpression::MultiplyExpression | confirmed | Constructor under an SEH frame. Calls the base constructor 0149adc0, stores MultiplyExpression::vftable into *this and returns this. |
| `01496420` | MultiplyExpression::GetTypeName | confirmed | Returns the literal string 'MultiplyExpression' (vtable slot [11]). |
| `01496460` | MultiplyExpression::~MultiplyExpression | confirmed | Destructor body. Stores MultiplyExpression::vftable into *this and calls the base destructor 0149ae80. |
| `014964b0` | MultiplyExpression::Evaluate | confirmed | Virtual slot [15] of MultiplyExpression. Card-set evaluation is not implemented: calls 012f5a60 with 'false', the MultiplyExpression.cpp path and line 0x12, th… |
| `014964d0` | MultiplyExpression::EvaluateValue | confirmed | Virtual slot [14] of MultiplyExpression. Traces '(Multiply', evaluates operands at this+0x1c and this+0x20 through vtable +0x38, requires integer type tags 2 o… |
| `014966b0` | MultiplyExpression::ScalarDeletingDtor | confirmed | Scalar deleting destructor, virtual slot [0] of MultiplyExpression. Stores the vtable, calls the base destructor 0149ae80, and frees this with _free when bit 0… |
| `01496630` | MultiplyExpression::Factory | likely | Allocates 0x24 bytes with operator new (00d833a6), runs MultiplyExpression::MultiplyExpression (014963a0) on the block and returns it; returns 0 when the alloc… |
| `01496690` | MultiplyExpression::RegisterFactory | likely | Registration initialiser. Calls 013ce130 with classID 0x26 and factory 01496630, then 013ce230 with the same pair. |

**NegateExpression** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01496050` | NegateExpression::NegateExpression | confirmed | Constructor under a stack-cookie frame. Calls the base constructor 0149adc0 and stores NegateExpression::vftable into *this, returning this. |
| `014960d0` | NegateExpression::GetTypeName | confirmed | Returns the literal string 'NegateExpression' (vtable slot [11]). |
| `01496110` | NegateExpression::~NegateExpression | confirmed | Destructor body. Stores NegateExpression::vftable into *this and calls the base destructor 0149ae80. |
| `01496160` | NegateExpression::Evaluate | confirmed | Virtual slot [15] of NegateExpression. Card-set evaluation is not implemented: calls 012f5a60 with 'false', the NegateExpression.cpp path and line 0x12, then r… |
| `01496180` | NegateExpression::EvaluateValue | confirmed | Virtual slot [14] of NegateExpression. Traces '(Negate', evaluates the operand at this+0x1c through vtable +0x38 and requires an integer result (type tag 2). O… |
| `01496340` | NegateExpression::ScalarDeletingDtor | confirmed | Scalar deleting destructor, virtual slot [0] of NegateExpression. Stores the vtable, calls the base destructor 0149ae80, and frees this with _free when bit 0 o… |
| `014962c0` | NegateExpression::Factory | likely | Allocates 0x24 bytes with operator new (00d833a6), runs NegateExpression::NegateExpression (01496050) on the block and returns it; returns 0 when the allocatio… |
| `01496320` | NegateExpression::RegisterFactory | likely | Registration initialiser. Calls 013ce130 with classID 0x32 and factory 014962c0, then 013ce230 with the same pair. |

**NotEqualToExpression** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01495cd0` | NotEqualToExpression::NotEqualToExpression | confirmed | Constructor under an SEH frame. Calls the base constructor 0149adc0, stores NotEqualToExpression::vftable into *this and returns this. |
| `01495d50` | NotEqualToExpression::GetTypeName | confirmed | Returns the literal string 'NotEqualToExpression' (vtable slot [11]). |
| `01495d90` | NotEqualToExpression::~NotEqualToExpression | confirmed | Destructor body. Stores NotEqualToExpression::vftable into *this and calls the base destructor 0149ae80 under an SEH frame. |
| `01495de0` | NotEqualToExpression::Evaluate | confirmed | Virtual slot [15] of NotEqualToExpression. Card-set evaluation is not implemented: calls 012f5a60 with the text 'false', the NotEqualToExpression.cpp path and … |
| `01495e00` | NotEqualToExpression::EvaluateValue | confirmed | Virtual slot [14] of NotEqualToExpression. Traces '(not equal', evaluates operand this+0x1c through vtable +0x38 and, when that succeeds, operand this+0x20 the… |
| `01495ff0` | NotEqualToExpression::ScalarDeletingDtor | confirmed | Scalar deleting destructor, virtual slot [0] of NotEqualToExpression. Stores the vtable, calls the base destructor 0149ae80, and frees this with _free when bit… |
| `01495f70` | NotEqualToExpression::Factory | likely | Allocates 0x24 bytes with operator new (00d833a6), runs NotEqualToExpression::NotEqualToExpression (01495cd0) on the block and returns it; returns 0 when the a… |
| `01495fd0` | NotEqualToExpression::RegisterFactory | likely | Registration initialiser. Calls 013ce130 with classID 0x22 and factory 01495f70, then 013ce230 with the same pair. |

**OrExpression** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01495660` | OrExpression::OrExpression | confirmed | Constructor under an SEH frame. Calls the base constructor 0149adc0, stores OrExpression::vftable into *this and returns this. |
| `014956e0` | OrExpression::GetTypeName | confirmed | Returns the literal string 'OrExpression' (vtable slot [11]). |
| `01495720` | OrExpression::~OrExpression | confirmed | Destructor body. Stores OrExpression::vftable into *this and calls the base destructor 0149ae80 under an SEH frame. |
| `01495770` | OrExpression::EvaluateValue | confirmed | Virtual slot [14] of OrExpression. Traces the opening text from DAT_018f896c, evaluates operand this+0x1c through vtable +0x38, and only when it is false evalu… |
| `01495860` | OrExpression::ScalarDeletingDtor | confirmed | Scalar deleting destructor, virtual slot [0] of OrExpression. Stores OrExpression::vftable, calls the base destructor 0149ae80, and frees this with _free when … |
| `014958c0` | OrExpression::Evaluate | confirmed | Virtual slot [15] of OrExpression. Traces '(gvt:or', evaluates both operands through their vtable +0x3c into two card lists, merges them into the output list (… |
| `014957e0` | OrExpression::Factory | likely | Allocates 0x24 bytes with operator new (00d833a6), runs OrExpression::OrExpression (01495660) on the block and returns it; returns 0 when the allocation fails. |
| `01495840` | OrExpression::RegisterFactory | likely | Registration initialiser. Calls 013ce130 with classID 0x1b and factory 014957e0, then 013ce230 with the same pair. |

**PileQueryNode** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01494390` | PileQueryNode::PileQueryNode | confirmed | Constructor. Runs QueryNode::QueryNode (01494290), stores PileQueryNode::vftable into *this, runs 012fa910 on the member at this+0x24, zeroes this+0x1c and thi… |
| `01494430` | PileQueryNode::GetTypeName | confirmed | Returns the literal string 'PileQueryNode' (vtable slot [11]). |
| `01494470` | PileQueryNode::~PileQueryNode | confirmed | Destructor body. Stores PileQueryNode::vftable, runs member cleanup 01300cd0, then calls the QueryNode destructor body 01493fd0. |
| `014946d0` | PileQueryNode::ScalarDeletingDtor | confirmed | Scalar deleting destructor, virtual slot [0] of PileQueryNode. Calls the destructor body 01494470, frees this with _free when bit 0 of the flag is set, and ret… |
| `01494a50` | PileQueryNode::GetCards | confirmed | Virtual slot [20] of PileQueryNode. Traces 'PileQueryNode::getCards', then switches on the pile selector at this+0x1c: 1 this, 2 Target1, 3 Target2, 5 Owner (w… |
| `01494e60` | PileQueryNode::EvaluateComparison | confirmed | Virtual slot [14] of PileQueryNode (also CWPileQueryNode slot [14]). Traces '(Pile Query Node', takes the number of gathered cards and compares it with the val… |
| `01494650` | PileQueryNode::Factory | likely | Allocates 0x30 bytes with operator new (00d833a6), runs PileQueryNode::PileQueryNode (01494390) on the block and returns it; returns 0 when the allocation fail… |
| `014946b0` | PileQueryNode::RegisterFactory | likely | Registration initialiser. Calls 013ce130 with classID 0x4c and factory 01494650, then 013ce230 with the same pair. |

**QueryNode** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01493e50` | QueryNode::Evaluate | confirmed | Virtual slot [15] of QueryNode. Traces '(gvt:QueryNode' through 01385610, traces ') => false' through 01384ee0 and returns 0 without producing cards. |
| `01493fd0` | QueryNode::~QueryNode | confirmed | Destructor body. Stores QueryNode::vftable, releases each element of the list at this+0x14 (for each element object at node+0x10, calls its scalar deleting des… |
| `01494110` | QueryNode::GetTypeName | confirmed | Returns the literal string 'QueryNode' (vtable slot [11], the class type name). |
| `01494290` | QueryNode::QueryNode | confirmed | Constructor under an SEH frame. Runs the base constructor 0149ab30, stores QueryNode::vftable into *this, runs 013128c0 and returns this. |
| `014942f0` | QueryNode::ScalarDeletingDtor | confirmed | Scalar deleting destructor, virtual slot [0] of QueryNode. Calls the destructor body 01493fd0, then frees this with _free when bit 0 of the flag argument is se… |
| `01494310` | QueryNode::Factory | likely | Allocates 0x1c bytes with operator new (00d833a6), runs the QueryNode constructor 01494290 on the block, and returns it; returns 0 when the allocation fails. |
| `01494370` | QueryNode::RegisterFactory | likely | Registration initialiser. Calls 013ce130 with classID 0x29 and the factory 01494310, then 013ce230 with the same pair. |

**QueryPlayabilityNode** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01492f10` | QueryPlayabilityNode::Evaluate | confirmed | Virtual slot [15] of QueryPlayabilityNode. Traces '(gvt QueryPlayabilityNode', builds the target1 candidate list from the kTarget1 source (01381460, with 00d8d… |
| `01493c20` | QueryPlayabilityNode::CanTarget1BePlayedOnTarget2 | confirmed | Virtual slot [14] of QueryPlayabilityNode. Fetches target1 and target2 cards through 01493970 with this+0x1c and this+0x20. If both exist, calls vtable +0x58 o… |

### `minigame.tcg.rules`

**CWActionNode** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01455e70` | CWActionNode::Ctor | confirmed | Constructor: calls base constructor 0149e640 with this in ECX under an SEH frame, then stores the CWActionNode vtable into *this and returns this. |
| `01455ef0` | CWActionNode::TypeName | confirmed | Virtual slot [11] of CWActionNode. Returns the literal string "CWActionNode" (mov eax, imm32; ret). |
| `01455f30` | CWActionNode::Dtor | confirmed | Non-deleting destructor: stores the CWActionNode vtable into *this, then calls base destructor 0149e710 under an SEH frame. Does not free the object. |
| `01455f80` | CWActionNode::GetNodeTarget | confirmed | Virtual slot [20] of CWActionNode. Traces 'CWActionNode::getNodeTarget %d' with its first argument, then forwards both arguments to 0149e820(param_1, param_2). |
| `01455fb0` | CWActionNode::Evaluate | confirmed | Virtual slot [14] of CWActionNode. Traces '(ZPActionNode'; if the byte at this+0x24 is non-zero it traces 'Setting Force!' and calls 01387670 with that byte. T… |
| `01456500` | CWActionNode::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): stores the CWActionNode vtable, calls base destructor 0149e710, and frees the object when flag bit 0 of param_2 i… |
| `01455f00` | CWActionNode::GetStaticDword | likely | Virtual slot [12] of CWActionNode. Returns the class static dword DAT_01cbe520 (mov eax, [01cbe520]; ret). |
| `01455f10` | CWActionNode::SetStaticDword | likely | Virtual slot [13] of CWActionNode. Stores its stack argument into DAT_01cbe520 and returns with ret 4. |
| `01456480` | CWActionNode::Factory | likely | Allocates 0x2c bytes with operator new 00d833a6; if null returns 0, otherwise runs the CWActionNode constructor 01455e70 on the block and returns the object. |
| `014564e0` | CWActionNode::RegisterFactory | likely | Registers factory 01456480 for classID 0x222ed: calls 013ce130(0x222ed, factory) and then 013ce230(0x222ed, factory). |

**CWActionPlayNode** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01455980` | CWActionPlayNode::Ctor | confirmed | Constructor: calls base constructor 0149d9a0 with this in ECX under an SEH frame, then stores the CWActionPlayNode vtable into *this and returns this. |
| `01455a00` | CWActionPlayNode::TypeName | confirmed | Virtual slot [11] of CWActionPlayNode. Returns the literal string "CWActionPlayNode" (mov eax, imm32; ret). |
| `01455a40` | CWActionPlayNode::Dtor | confirmed | Non-deleting destructor: stores the CWActionPlayNode vtable into *this, then calls base destructor 0149da60 under an SEH frame. Does not free the object. |
| `01455c30` | CWActionPlayNode::Evaluate | confirmed | Virtual slot [14] of CWActionPlayNode. Traces '(ZPActionPlayNode'. If this+0x10 is 0x422 it traces 'pile == Target3PlayArea' and uses selector 0x36; if 0x423 i… |
| `01455e10` | CWActionPlayNode::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): stores the CWActionPlayNode vtable, calls base destructor 0149da60, and frees the object when flag bit 0 of param… |
| `01455a10` | CWActionPlayNode::GetStaticDword | likely | Virtual slot [12] of CWActionPlayNode. Returns the class static dword DAT_01cbe51c (mov eax, [01cbe51c]; ret). |
| `01455a20` | CWActionPlayNode::SetStaticDword | likely | Virtual slot [13] of CWActionPlayNode. Stores its stack argument into DAT_01cbe51c and returns with ret 4. |
| `01455d90` | CWActionPlayNode::Factory | likely | Allocates 0x1c bytes with operator new 00d833a6; if null returns 0, otherwise runs the CWActionPlayNode constructor 01455980 and returns the object. |
| `01455df0` | CWActionPlayNode::RegisterFactory | likely | Registers factory 01455d90 for classID 0x222f6: calls 013ce130(0x222f6, factory) and then 013ce230(0x222f6, factory). |

**CWEachExpression** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014556d0` | CWEachExpression::Ctor | confirmed | Constructor: calls the base constructor 0149ba60 with this in ECX under an SEH frame, then stores the CWEachExpression vtable into *this and returns this. |
| `01455750` | CWEachExpression::TypeName | confirmed | Virtual slot [11] of CWEachExpression. Returns the literal string "CWEachExpression" (mov eax, imm32; ret). |
| `01455790` | CWEachExpression::Dtor | confirmed | Non-deleting destructor: stores the CWEachExpression vtable into *this, then calls base destructor 0149bb20 under an SEH frame. Does not free the object. |
| `014557e0` | CWEachExpression::GetCards | confirmed | Virtual slot [20] of CWEachExpression. Traces 'ZPEachExpression::getCardList' via 013846b0, then calls 0149c170 with this in ECX and its two stack arguments (p… |
| `01455920` | CWEachExpression::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): stores the CWEachExpression vtable, calls base destructor 0149bb20, and frees the object when flag bit 0 of param… |
| `01455760` | CWEachExpression::GetStaticDword | likely | Virtual slot [12] of CWEachExpression. Returns the class static dword DAT_01cbe518 (mov eax, [01cbe518]; ret). |
| `01455770` | CWEachExpression::SetStaticDword | likely | Virtual slot [13] of CWEachExpression. Stores its stack argument into DAT_01cbe518 and returns with ret 4. |
| `014558a0` | CWEachExpression::Factory | likely | Allocates 0x2c bytes with operator new 00d833a6; if null returns 0, otherwise runs the CWEachExpression constructor 014556d0 on the block and returns the objec… |
| `01455900` | CWEachExpression::RegisterFactory | likely | Registers factory 014558a0 for classID 0x222f1: calls 013ce130(0x222f1, factory) and then 013ce230(0x222f1, factory). |

**CWLastActionQueryNode** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01454ae0` | CWLastActionQueryNode::Ctor | confirmed | Constructor: calls base constructor 01498290 with this under an SEH frame, then stores the CWLastActionQueryNode vtable into *this and returns this. |
| `01454b60` | CWLastActionQueryNode::TypeName | confirmed | Virtual slot [11] of CWLastActionQueryNode. Returns the literal string "CWLastActionQueryNode" (mov eax, imm32; ret). |
| `01454ba0` | CWLastActionQueryNode::Dtor | confirmed | Non-deleting destructor: stores the CWLastActionQueryNode vtable into *this, then calls base destructor 01498350 under an SEH frame. Does not free the object. |
| `01454d00` | CWLastActionQueryNode::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): stores the CWLastActionQueryNode vtable, calls base destructor 01498350, and frees the object when flag bit 0 of … |
| `01454d60` | CWLastActionQueryNode::Evaluate | confirmed | Virtual slot [14] of CWLastActionQueryNode. Traces the container and eventID, compares the environment event 01383bd0() with mEvent at this+0x1c (mismatch: tra… |
| `014554b0` | CWLastActionQueryNode::GetCards | confirmed | Virtual slot [20] of CWLastActionQueryNode. If param_2 (event card) is null it traces 'Event Card is null!' and returns 0. Otherwise it walks the entry list at… |
| `01454b70` | CWLastActionQueryNode::GetStaticDword | likely | Virtual slot [12] of CWLastActionQueryNode. Returns the global dword DAT_01cbe514 (mov eax, [01cbe514]; ret). |
| `01454b80` | CWLastActionQueryNode::SetStaticDword | likely | Virtual slot [13] of CWLastActionQueryNode. Stores its stack argument into DAT_01cbe514 and returns with ret 4. |
| `01454c80` | CWLastActionQueryNode::Factory | likely | Allocates 0x24 bytes with operator new 00d833a6; if null returns 0, otherwise runs CWLastActionQueryNode::Ctor 01454ae0 and returns the object. |
| `01454ce0` | CWLastActionQueryNode::RegisterFactory | likely | Registers factory 01454c80 for classID 0x222f3: calls 013ce130(0x222f3, factory) and then 013ce230(0x222f3, factory). |

**CWMacroNode** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014547a0` | CWMacroNode::Ctor | confirmed | Constructor: calls the base constructor 01423400 with this under an SEH frame, then stores the CWMacroNode vtable into *this and returns this. |
| `01454820` | CWMacroNode::TypeName | confirmed | Virtual slot [11] of CWMacroNode. Returns the literal string "CWMacroNode" (mov eax, imm32; ret). |
| `01454860` | CWMacroNode::Dtor | confirmed | Non-deleting destructor: stores the CWMacroNode vtable into *this, then calls the base destructor 014234c0 under an SEH frame. Does not free the object. |
| `01454a80` | CWMacroNode::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): stores the CWMacroNode vtable, calls base destructor 014234c0, and frees the object when flag bit 0 of param_2 is… |
| `01454830` | CWMacroNode::GetStaticDword | likely | Virtual slot [12] of CWMacroNode. Returns the global dword DAT_01cbe510 (mov eax, [01cbe510]; ret). |
| `01454840` | CWMacroNode::SetStaticDword | likely | Virtual slot [13] of CWMacroNode. Stores its stack argument into DAT_01cbe510 and returns with ret 4. |
| `014548b0` | CWMacroNode::TracePredicateTrue | likely | Virtual slot [15] of CWMacroNode. Writes the trace text '(gvt:CWMacroNode' and ') => true' through 01385610 and 01384ee0 and returns 1 without testing anything. |
| `01454a00` | CWMacroNode::Factory | likely | Allocates 0x14 bytes with operator new 00d833a6; if null returns 0, otherwise runs the CWMacroNode constructor 014547a0 on the block and returns the object. |
| `01454a60` | CWMacroNode::RegisterFactory | likely | Registers factory 01454a00 for classID 0x222ec: calls 013ce130(0x222ec, factory) and then 013ce230(0x222ec, factory). |

**CWMatchCommand_LaunchGame** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014595a0` | CWMatchCommand_LaunchGame::Ctor | confirmed | Constructor: calls base constructor 014b4c90 with this in ECX under an SEH frame, then stores the CWMatchCommand_LaunchGame vtable 018ee094 into *this and retu… |
| `01459620` | CWMatchCommand_LaunchGame::TypeName | confirmed | Virtual slot [11] of CWMatchCommand_LaunchGame. Returns the literal string "CWMatchCommand_LaunchGame" (mov eax, imm32; ret). |
| `01459660` | CWMatchCommand_LaunchGame::Dtor | confirmed | Non-deleting destructor: stores the command vtable into *this, then calls base destructor 014b4d50 under an SEH frame. Does not free the object. |
| `01459810` | CWMatchCommand_LaunchGame::CopyFieldsTo | confirmed | Virtual slot [18] (CopyFieldsTo). Dynamic-casts the target to CWMatchCommand_LaunchGame from NetworkCommand through 00d8d382; if the cast fails it asserts 'clo… |
| `01459970` | CWMatchCommand_LaunchGame::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): stores the command vtable, calls base destructor 014b4d50, and frees the object when flag bit 0 of param_2 is set… |
| `01459630` | CWMatchCommand_LaunchGame::GetStaticDword | likely | Virtual slot [12] of CWMatchCommand_LaunchGame. Returns the class static dword DAT_01cbe550 (mov eax, [01cbe550]; ret). |
| `01459640` | CWMatchCommand_LaunchGame::SetStaticDword | likely | Virtual slot [13] of CWMatchCommand_LaunchGame. Stores its stack argument into DAT_01cbe550 and returns with ret 4. |
| `014598f0` | CWMatchCommand_LaunchGame::Factory | likely | Allocates 0xc bytes with operator new 00d833a6; if null returns 0, otherwise runs the constructor 014595a0 on the block and returns the object. |
| `01459950` | CWMatchCommand_LaunchGame::RegisterFactory | likely | Registers factory 014598f0 for classID 0x222e8: calls 013ce130(0x222e8, factory) and then 013ce230(0x222e8, factory). |

**CWPileQueryNode** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01454440` | CWPileQueryNode::Ctor | confirmed | Constructor: calls the PileQueryNode base constructor 01494390 with this in ECX under an SEH frame, then stores the CWPileQueryNode vtable 018ed82c into *this … |
| `014544c0` | CWPileQueryNode::TypeName | confirmed | Virtual slot [11] of CWPileQueryNode. Returns the literal string "CWPileQueryNode" (mov eax, imm32; ret). |
| `01454500` | CWPileQueryNode::Dtor | confirmed | Non-deleting destructor: stores the CWPileQueryNode vtable into *this, then calls the PileQueryNode base destructor 01494470 under an SEH frame. Does not free … |
| `01454660` | CWPileQueryNode::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): stores the CWPileQueryNode vtable into *this, calls the PileQueryNode base destructor 01494470, and frees the obj… |
| `014546c0` | CWPileQueryNode::GetCards | confirmed | Virtual slot [20] of CWPileQueryNode. Traces 'CWPileQueryNode::getCards %d' with the selector at this+0x1c, then switches on it: 0x3a and 0x3b take 01383ec0(0x… |
| `014544d0` | CWPileQueryNode::GetStaticDword | likely | Virtual slot [12] of CWPileQueryNode. Returns the class static dword DAT_01cbe50c (mov eax, [01cbe50c]; ret), with no object access. |
| `014544e0` | CWPileQueryNode::SetStaticDword | likely | Virtual slot [13] of CWPileQueryNode. Stores its stack argument into DAT_01cbe50c and returns with ret 4. |
| `014545e0` | CWPileQueryNode::Factory | likely | Allocates 0x30 bytes with operator new 00d833a6; if the allocation is null returns 0, otherwise runs the CWPileQueryNode constructor 01454440 on it and returns… |
| `01454640` | CWPileQueryNode::RegisterFactory | likely | Registers factory 014545e0 for classID 0x222f0: calls 013ce130(0x222f0, factory) and then 013ce230(0x222f0, factory). |

**CWSumExpression** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01454360` | CWSumExpression::getCardList | confirmed | Virtual slot [20] of CWSumExpression. Traces 'ZPSumExpression::getCardList' via 013846b0, then switches on the selector at this+0x24: 0x3a and 0x3b take 01383e… |

**Engine::FRTWDeckValidator** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01537e60` | Engine::FRTWDeckValidator::FRTWDeckValidator | confirmed | Fastcall constructor: calls the base constructor 014393d0 (its vtable store is Engine::DeckValidatorBase::vftable), then stores Engine::FRTWDeckValidator::vfta… |
| `01537f10` | Engine::FRTWDeckValidator::~FRTWDeckValidator | confirmed | Scalar deleting destructor (vtable slot [0]): stores Engine::FRTWDeckValidator::vftable, calls base destructor 01439430, then _free(this) only when bit 0 of th… |
| `01537ec0` | Engine::FRTWDeckValidator::~FRTWDeckValidator | likely | Non-deleting destructor body (fastcall): stores Engine::FRTWDeckValidator::vftable at this, keeps this in an EH frame local, calls base destructor 01439430 and… |

**EvaluationEnvironment** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013812c0` | EvaluationEnvironment::InitFromEnvironment | confirmed | Thiscall with one stack argument (ret 4, source environment). Asserts the source is non-null (EvaluationEnvironment.cpp line 0x66, text 'environment'); copies … |
| `01381380` | EvaluationEnvironment::GetGame | confirmed | Fastcall (ret 0). Returns this+4 (mGame). When it is zero it first calls 012f5a60 with the assert text 'mGame' and line 0x2bd. |
| `013813e0` | EvaluationEnvironment::GetCachedValid8 | confirmed | Fastcall (ret 0). Asserts this+4 (mGame) is non-null (EvaluationEnvironment.cpp line 0x3df) when it is zero. If the cached dword at this+8 is zero it calls the… |
| `01385460` | EvaluationEnvironment::Dtor | confirmed | Fastcall (no stack arguments). Stores EvaluationEnvironment::vftable at +0, then erases the whole range +0x80..+0x84 of the container at +0x7c (through 01383cf… |
| `013855b0` | EvaluationEnvironment::GetClassName | confirmed | Virtual slot [11] (+0x2c) of EvaluationEnvironment::vftable, returns the constant string 'EvaluationEnvironment' (6 bytes: mov eax, imm; ret). |
| `01385940` | EvaluationEnvironment::EvaluationEnvironment | confirmed | Fastcall (no stack arguments, returns this). Stores PersistentBase::vftable, PersistentComponent::vftable and then EvaluationEnvironment::vftable at +0, zeroes… |
| `01385a30` | EvaluationEnvironment::ScalarDeletingDtor | confirmed | Scalar deleting destructor, thiscall with a flag byte (ret 4). Calls the destructor body 01385460 on this, then calls _free(this) when bit 0 of the flag is set… |
| `01381490` | EvaluationEnvironment::GetValidField0c | likely | Fastcall (ret 0, returns int). Asserts this+4 (mGame) is non-null (line 0x2bd) when it is zero, then calls 01357a00 with the dword at this+0xc. It returns that… |
| `01381d70` | EvaluationEnvironment::CastCheckField10 | likely | Fastcall (ret 0). Asserts this+4 (mGame) is non-null (line 0x2bd) when it is zero, calls 01357a00 with the dword at this+0x10, then calls 00d8d382 (RTTI dynami… |
| `01385a50` | EvaluationEnvironment::CollectCardsKey28 | likely | Thiscall with one stack argument (out list, returns it). Asserts mGame non-null (line 0x158) when this+4 is zero, calls 013833c0 with key 0x28 (chain find into… |

### `net.tcg`

**Card** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0139bcb0` | Card::TypeName | likely | Returns the address of the constant string DAT_018cd818 (bytes 'Card', NUL-terminated); no arguments or state. |
| `0139bcc0` | Card::StaticDwordGet | likely | Returns the global dword DAT_01cbdbd4 (mov eax,[0x1cbdbd4]; ret); no arguments. |
| `0139bcd0` | Card::StaticDwordSet | likely | Stores its single stack argument into DAT_01cbdbd4 and returns with ret 4. |
| `0139bcf0` | Card::DeletingDtorThunk_0x48 | likely | Thunk: subtracts 0x48 from ECX (the +0x48 base subobject) and jumps to the Card scalar deleting destructor 0139d1e0. |
| `0139bd00` | Card::DeletingDtorThunk_0x58 | likely | Thunk: subtracts 0x58 from ECX (the +0x58 base subobject) and jumps to the Card scalar deleting destructor 0139d1e0. |
| `0139bd20` | Card::DeletingDtorThunk_0x8 | likely | Thunk: subtracts 8 from ECX (the secondary command vtable subobject at +8) and jumps to the Card scalar deleting destructor 0139d1e0. |
| `0139d200` | Card::Factory | likely | Allocates 0x140 bytes through operator new (00d833a6); when non-null constructs the Card through 0139d010 and returns the object pointer + 8; returns 0 when al… |
| `0139d280` | Card::RegisterFactory | likely | Static initializer: calls 013ce130 with classID 0xa8 and the factory 0139d200, then 013ce230 with the same classID and factory. |

**CommandObject_RemoveInstalledAction** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01500300` | CommandObject_RemoveInstalledAction::Read | confirmed | Secondary vtable slot [9] (+0x24). Calls slot [4] (+0x10) with the stream and an out value (assert CommandObjectRemoveInstalledAction.cpp line 0x5a on failure)… |
| `01500370` | CommandObject_RemoveInstalledAction::Write | confirmed | Secondary vtable slot [10] (+0x28). Calls slot [2] (+0x08), writes the trace strings through 013c26e0 and 0142acd0, writes the integer at object+0x124 with 013… |

**CommandObject_RemoveKeyword** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014fff10` | CommandObject_RemoveKeyword::Read | confirmed | Secondary vtable slot [9] (+0x24). Calls slot [4] (+0x10) with the stream and an out value (assert CommandObjectRemoveKeyword.cpp line 0x42 on failure), calls … |
| `014fffa0` | CommandObject_RemoveKeyword::Write | confirmed | Secondary vtable slot [10] (+0x28). Calls slot [2] (+0x08), writes the trace strings through 013c26e0 and 0142acd0, writes the label "ValueData: " and calls sl… |
| `01500190` | CommandObject_RemoveKeyword::Create | likely | Allocates 0x130 bytes with 00d833a6; when non-null it constructs the object with 01500040 and returns the block plus 8; returns 0 otherwise. |
| `01500210` | CommandObject_RemoveKeyword::RegisterFactory | likely | Pushes the id 0xb4 and the factory 01500190 and calls 013ce130 (cdecl), then calls 013ce230 with the returned registry as ECX. Returns void. |

**CommandObject_RemoveProperty** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ff810` | CommandObject_RemoveProperty::Write | confirmed | Secondary vtable slot [10] (+0x28). Calls slot [2] (+0x08), writes the trace strings through 013c26e0 and 0142acd0, writes the label "ValueData: " and calls sl… |
| `014ffcf0` | CommandObject_RemoveProperty::Create | likely | Allocates 0x140 bytes with 00d833a6; when non-null it constructs the object with 014ffb60 and returns the block plus 8; returns 0 otherwise. |
| `014ffd70` | CommandObject_RemoveProperty::RegisterFactory | likely | Pushes the id 0x175 and the factory 014ffcf0 and calls 013ce130 (cdecl), then calls 013ce230 with the returned registry as ECX. Returns void. |

**CommandObject_RemoveSuppressEffect** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ff290` | CommandObject_RemoveSuppressEffect::Read | confirmed | Secondary vtable slot [9] (+0x24). Calls slot [4] (assert CommandObjectRemoveSuppressEffect.cpp line 0x43 on failure), calls 0142a880 (assert 0x46), calls slot… |
| `014ff320` | CommandObject_RemoveSuppressEffect::Write | confirmed | Secondary vtable slot [10] (+0x28). Calls slot [2] (+0x08), writes the trace strings through 013c26e0 and 0142acd0, writes the label "ValueData: " and calls sl… |
| `014ff510` | CommandObject_RemoveSuppressEffect::Create | likely | Allocates 0x130 bytes with 00d833a6; when non-null it constructs the object with 014ff3c0 and returns the block plus 8; returns 0 otherwise. |
| `014ff590` | CommandObject_RemoveSuppressEffect::RegisterFactory | likely | Pushes the factory 014ff510 and the id 0xda and calls 013ce130 (cdecl), then calls 013ce230 with the returned registry as ECX. Returns void. |

**CommandObject_RemoveToken** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014feb50` | CommandObject_RemoveToken::Read | confirmed | Secondary vtable slot [9] (+0x24). Calls slot [4] (assert CommandObjectRemoveToken.cpp line 0x95 on failure), calls 0142a880 (assert 0x98), calls slot 9 (+0x24… |
| `014fecb0` | CommandObject_RemoveToken::Write | confirmed | Secondary vtable slot [10] (+0x28). Calls slot [2] (+0x08), writes the trace strings through 013c26e0 and 0142acd0; writes the label "Value DAta: " and calls s… |
| `014ff070` | CommandObject_RemoveToken::Create | likely | Allocates 0x13c bytes with 00d833a6; when non-null it constructs the object with 014fedb0 and returns the block plus 8; returns 0 otherwise. |
| `014ff0f0` | CommandObject_RemoveToken::RegisterFactory | likely | Pushes the id 0xb8 and the factory 014ff070 and calls 013ce130 (cdecl), then calls 013ce230 with the returned registry as ECX. Returns void. |

**CommandObject_Reshuffle** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014fe540` | CommandObject_Reshuffle::Read | confirmed | Secondary vtable slot [9] (+0x24). Calls slot [4] (+0x10) with the stream and an out value (assert CommandObjectReshuffle.cpp line 0x37 on failure), calls 0142… |
| `014fe5a0` | CommandObject_Reshuffle::Write | confirmed | Secondary vtable slot [10] (+0x28). Calls slot [2] (+0x08) with the stream, writes the trace strings through 013c26e0 and 0142acd0 (no integers), then calls sl… |
| `014fe790` | CommandObject_Reshuffle::Create | likely | Allocates 0x124 bytes with 00d833a6; when non-null it constructs the object with 014fe620 and returns the block plus 8; returns 0 otherwise. |
| `014fe810` | CommandObject_Reshuffle::RegisterFactory | likely | Pushes the id 0x137 and the factory 014fe790 and calls 013ce130 (cdecl), then calls 013ce230 with the returned registry as ECX. Returns void. |

**CommandObject_Reveal** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014fdc40` | CommandObject_Reveal::Read | confirmed | Secondary vtable slot [9] (+0x24). Calls slot [4] (+0x10) with the stream and an out value (assert CommandObjectReveal.cpp line 0x84 on failure), calls 0142a88… |
| `014fdcb0` | CommandObject_Reveal::Write | confirmed | Secondary vtable slot [10] (+0x28). Calls slot [2] (+0x08) with the stream, writes the trace strings through 013c26e0 and 0142acd0 (no integers), then calls sl… |
| `014fdee0` | CommandObject_Reveal::Create | likely | Allocates 0x124 bytes with 00d833a6; when non-null it constructs the object with 014fdd30 and returns the block plus 8; returns 0 otherwise. |
| `014fdf60` | CommandObject_Reveal::RegisterFactory | likely | Pushes the factory 014fdee0 and the id 0xd0 and calls 013ce130 (cdecl), then calls 013ce230 with the returned registry as ECX. Returns void. |

**CommandObject_SendCardIntroductions** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014fd890` | CommandObject_SendCardIntroductions::Read | confirmed | Secondary vtable slot [9] (+0x24). Calls slot [4] (+0x10) with the stream and an out value (assert CommandObjectSendCardIntroductions.cpp line 0x37 on failure)… |
| `014fdad0` | CommandObject_SendCardIntroductions::Create | likely | Allocates 0x124 bytes with 00d833a6; when non-null it constructs the object with 014fd970 and returns the block plus 8; returns 0 otherwise. |
| `014fdb50` | CommandObject_SendCardIntroductions::RegisterFactory | likely | Pushes the id 0xde and the factory 014fdad0 and calls 013ce130 (cdecl), then calls 013ce230 with the returned registry as ECX. Returns void. |

**CommandObject_SetPhaseDialog** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014fd0b0` | CommandObject_SetPhaseDialog::Read | confirmed | Secondary vtable slot [9] (+0x24). Calls slot [4] (+0x10) with the stream and an out value (assert CommandObjectSetPhaseDialog.cpp line 0x24 on failure), calls… |
| `014fd310` | CommandObject_SetPhaseDialog::Create | likely | Allocates 0x130 bytes with 00d833a6; when non-null it constructs the object with 014fd1f0 and returns the block plus 8; returns 0 otherwise. |
| `014fd390` | CommandObject_SetPhaseDialog::RegisterFactory | likely | Pushes the id 0x180 and the factory 014fd310 and calls 013ce130 (cdecl), then calls 013ce230 with the returned registry as ECX. Returns void. |

**CommandObject_SetProperty** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014fc470` | CommandObject_SetProperty::Write | confirmed | Secondary vtable slot [10] (+0x28). Calls slot [2] (+0x08) with the stream, writes the trace strings through 013c26e0 and 0142acd0, writes object+0x124 with 01… |
| `014fc6e0` | CommandObject_SetProperty::Create | likely | Allocates 0x138 bytes with 00d833a6; when non-null it constructs the object with 014fc520 and returns the block plus 8; returns 0 otherwise. |
| `014fc760` | CommandObject_SetProperty::RegisterFactory | likely | Pushes the id 0x159 and the factory 014fc6e0 and calls 013ce130 (cdecl), then calls 013ce230 with the returned registry as ECX. Returns void. |

**CommandObject_Shuffle** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014fbe30` | CommandObject_Shuffle::Read | confirmed | Secondary vtable slot [9] (+0x24). Calls slot [4] (assert CommandObjectShuffle.cpp line 0x54 on failure), calls 0142a880 (assert line 0x57), reads the play-are… |
| `014fbf20` | CommandObject_Shuffle::Write | confirmed | Secondary vtable slot [10] (+0x28). Calls slot [2] (+0x08), writes the trace strings through 013c26e0 and 0142acd0, writes object+0x124 with 013c23f0, then wri… |
| `014fc1b0` | CommandObject_Shuffle::Create | likely | Allocates 300 bytes (0x12c) with 00d833a6; when non-null it constructs the object with 014fbfe0 (the no-argument Shuffle constructor) and returns the block plu… |
| `014fc230` | CommandObject_Shuffle::RegisterFactory | likely | Pushes the id 0xb6 and the factory 014fc1b0 and calls 013ce130 (cdecl), then calls 013ce230 with the returned registry as ECX. Returns void. |

**CommandObject_SuppressGameText** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014fba10` | CommandObject_SuppressGameText::Read | confirmed | Secondary vtable slot [9] (+0x24). Calls slot [4] (+0x10) with the stream and an out value (assert CommandObjectSuppressGameText.cpp line 0x33 on failure), cal… |
| `014fba70` | CommandObject_SuppressGameText::Write | confirmed | Secondary vtable slot [10] (+0x28). Calls slot [2] (+0x08) with the stream, writes the trace strings through 013c26e0 and 0142acd0 (no integers), then calls sl… |
| `014fbc40` | CommandObject_SuppressGameText::Create | likely | Allocates 0x124 bytes with 00d833a6; when non-null it constructs the object with 014fbaf0 and returns the block plus 8 (the secondary subobject); returns 0 oth… |
| `014fbcc0` | CommandObject_SuppressGameText::RegisterFactory | likely | Pushes the id 0x147 and the factory 014fbc40 and calls 013ce130 (cdecl), then calls 013ce230 with the returned registry as ECX. Returns void. |

**CommandObject_UIEffect** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014fb640` | CommandObject_UIEffect::Read | confirmed | Secondary vtable slot [9] (+0x24). Calls slot [4] (+0x10) with the stream and an out value, asserting CommandObjectUIEffect.cpp line 0x4e on failure; calls 014… |
| `014fb6d0` | CommandObject_UIEffect::Write | confirmed | Secondary vtable slot [10] (+0x28). Calls slot [2] (+0x08) with the stream, writes trace strings through 013c26e0 and 0142acd0, writes the integer at object+0x… |
| `014fb850` | CommandObject_UIEffect::Create | likely | Allocates 0x128 bytes with 00d833a6; when non-null it constructs the object with 014fb760 and returns the block plus 8 (the secondary subobject); returns 0 oth… |
| `014fb8d0` | CommandObject_UIEffect::RegisterFactory | likely | Pushes the factory 014fb850 and the id 0x1d8 and calls 013ce130 (cdecl), then calls 013ce230 with the returned registry as ECX. Returns void. |

**CommandObject_UnInstallActionMap** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014fb0b0` | CommandObject_UnInstallActionMap::Read | confirmed | Secondary vtable slot [9] (+0x24), called on the secondary subobject (object = this-8). Calls slot [4] (+0x10) with the stream and an out value, asserting Comm… |
| `014fb1c0` | CommandObject_UnInstallActionMap::Write | confirmed | Secondary vtable slot [10] (+0x28), called on the secondary subobject. Calls slot [2] (+0x08) with the stream, writes the trace strings through 013c26e0 (a no-… |
| `014fb3b0` | CommandObject_UnInstallActionMap::Create | likely | Allocates 300 bytes (0x12c) with 00d833a6; when the allocation is non-null it constructs the object in it with 014fb290 and returns the block plus 8 (the secon… |
| `014fb430` | CommandObject_UnInstallActionMap::RegisterFactory | likely | Pushes the id 0xcb and the factory 014fb3b0 and calls 013ce130 (registry lookup, cdecl), then calls 013ce230 with the returned registry as ECX. Returns void. |

**DeckCommand** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01442180` | DeckCommand::DeckCommand | confirmed | Fastcall constructor (this in ECX, returns this). Calls the base constructor 0140c090 on this (ECX unchanged, the same base constructor LoginCommand uses) and … |
| `01442200` | DeckCommand::TypeName | confirmed | Virtual slot 11 with no arguments. Returns the address of the literal NetworkCommand (018e4d44), not a DeckCommand string. |
| `01442240` | DeckCommand::~DeckCommand | confirmed | Destructor body (fastcall, this in ECX, no deleting part). Stores DeckCommand::vftable at +0, calls the base destructor 0140c160 on this and returns. |
| `014422a0` | DeckCommand::Clone | confirmed | Virtual slot 17 (fastcall, this in ECX). Allocates 4 bytes with operator new (00d833a6); when non-null default-constructs the block with 01442180; then calls s… |
| `01442310` | DeckCommand::CopyFieldsTo | confirmed | Virtual slot 18 (one stack argument, the destination NetworkCommand). Casts it with the RTTI dynamic_cast 00d8d382 from NetworkCommand to DeckCommand; when the… |
| `01442360` | DeckCommand::Read | confirmed | Virtual slot 9 (thiscall, one stack argument). Calls slot 4 on this with (param_2, &param_2) and returns 0 when that fails; calls base-class read 0140c2a0 with… |
| `014423a0` | DeckCommand::Write | confirmed | Virtual slot 10 (thiscall, one stack argument). Calls slot 2 on this with param_2, calls 013c26e0 with the text Starting DeckCommand (a no-op that returns 1, s… |
| `01442470` | DeckCommand::ScalarDeletingDestructor | confirmed | Scalar deleting destructor (thiscall, one stack argument, ret 4). Stores DeckCommand::vftable at +0, calls the base destructor 0140c160 on this, then calls _fr… |
| `014424d0` | DeckCommand::DumpText | confirmed | Virtual slot 19 (thiscall, one stack argument, the std::string that receives text). Appends the 13-byte line Deck Command with a newline through 01241650, then… |
| `01442500` | DeckCommand::AppendDisplayName | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads ECX from the argument (the std::string receiving text) and appends the 12-byte literal Deck Command (018ea7e… |
| `014423f0` | DeckCommand::Create | likely | Factory with no arguments. Calls operator new with 4 bytes (00d833a6); returns 0 when that returns null, otherwise default-constructs the block with 01442180 a… |

**DeckCommand_AddOnlineDeck** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013cff10` | DeckCommand_AddOnlineDeck::DeckCommand_AddOnlineDeck | confirmed | Constructor: calls 01442180, stores DeckCommand_AddOnlineDeck::vftable, and clears the pointer at +0x08; returns this. |
| `013cff90` | DeckCommand_AddOnlineDeck::CommandName | confirmed | Virtual slot 11: returns the address of the literal DeckCommand_AddOnlineDeck. |
| `013cffd0` | DeckCommand_AddOnlineDeck::~DeckCommand_AddOnlineDeck | confirmed | Destructor body: stores the vtable, frees the pointer at +0x08 when non-null, then calls 01442240. |
| `013d0120` | DeckCommand_AddOnlineDeck::Clone | confirmed | Virtual slot 17: allocates 0x0c bytes with 00d833a6, constructs an AddOnlineDeck (013cff10) in them, then calls slot 18 of this (013d0190) with the new object;… |
| `013d0190` | DeckCommand_AddOnlineDeck::CopyInto | confirmed | Virtual slot 18: dynamic-casts the argument (NetworkCommand to AddOnlineDeck) through 00d8d382, asserts the cast result with the expression clone at DeckComman… |
| `013d0380` | DeckCommand_AddOnlineDeck::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the body 013cffd0, then calls _free on this when bit 0 of the second argument is set; returns this. |
| `013d0460` | DeckCommand_AddOnlineDeck::DebugDump | confirmed | Virtual slot 19: writes Add Online Deck Deck Command through 01241650, calls 014424d0 on the argument, formats SerializedDeckLength : %d from this+0x04 with _s… |
| `013d0520` | DeckCommand_AddOnlineDeck::EmitDebugName | confirmed | Virtual slot 20: passes the literal Add Online Deck Deck Command (length 0x1c) to 01241650. |
| `013d0200` | DeckCommand_AddOnlineDeck::Deserialize | likely | Virtual slot 9: reads a header through slot 4, calls 01442360 with the argument, reads an integer into this+0x04 with 013c24d0, allocates that many bytes into … |
| `013d0280` | DeckCommand_AddOnlineDeck::Serialize | likely | Virtual slot 10: calls slot 2 with its argument, writes trace text through 013c26e0 (a no-op), calls 014423a0, appends the length at this+0x04 with SerialAppen… |
| `013d0300` | DeckCommand_AddOnlineDeck::Create | likely | Allocates 0x0c bytes with 00d833a6; when non-null constructs an AddOnlineDeck with 013cff10 and returns it, otherwise returns 0. |

**DeckCommand_DeleteOnlineDeck** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013cfb30` | DeckCommand_DeleteOnlineDeck::DeckCommand_DeleteOnlineDeck | confirmed | Constructor: calls 01442180, stores DeckCommand_DeleteOnlineDeck::vftable, and sets the DeckID string at +0x04 to empty (capacity 15, size 0, first byte 0); re… |
| `013cfbc0` | DeckCommand_DeleteOnlineDeck::CommandName | confirmed | Virtual slot 11: returns the address of the literal DeckCommand_DeleteOnlineDeck. |
| `013cfc00` | DeckCommand_DeleteOnlineDeck::~DeckCommand_DeleteOnlineDeck | confirmed | Destructor body: stores the vtable, frees the DeckID heap buffer at +0x08 when capacity exceeds 15, resets the string, then calls 01442240. |
| `013cfc80` | DeckCommand_DeleteOnlineDeck::Clone | confirmed | Virtual slot 17: allocates 0x20 bytes, constructs a DeleteOnlineDeck (013cfb30) in them, then calls slot 18 of this (013cfdb0) with the new object, which copie… |
| `013cfd70` | DeckCommand_DeleteOnlineDeck::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the body 013cfc00, then calls _free on this when bit 0 of the second argument is set; returns this. |
| `013cfe10` | DeckCommand_DeleteOnlineDeck::DebugDump | confirmed | Virtual slot 19: writes the text Delete Online Deck Deck Command through 01241650, calls 014424d0 on the argument, formats DeckID: followed by the +0x04 string… |
| `013cfeb0` | DeckCommand_DeleteOnlineDeck::EmitDebugName | confirmed | Virtual slot 20: passes the literal Delete Online Deck Deck Command (length 0x1f) to 01241650. |
| `013cfa80` | DeckCommand_DeleteOnlineDeck::Deserialize | likely | Virtual slot 9: reads a header through slot 4, calls 01442360, reads the DeckID string into the string at +0x04 with SerialReadString 013c28d0, then calls slot… |
| `013cfad0` | DeckCommand_DeleteOnlineDeck::Serialize | likely | Virtual slot 10: calls slot 2 with its argument, writes the trace text through 013c26e0 (a no-op), calls 014423a0, writes the DeckID string at +0x04 with Seria… |
| `013cfcf0` | DeckCommand_DeleteOnlineDeck::Create | likely | Allocates 0x20 bytes with 00d833a6; when non-null constructs a DeleteOnlineDeck with 013cfb30 and returns it, otherwise returns 0. |
| `013cfdb0` | DeckCommand_DeleteOnlineDeck::CopyInto | likely | Virtual slot 18: dynamic-casts the argument (NetworkCommand to DeleteOnlineDeck) through 00d8d382, asserts the cast result with the expression clone at DeckCom… |

**GameCommand_DrawCards** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01424b90` | GameCommand_DrawCards::ReleaseMembers | confirmed | Destructor body: stores GameCommand_DrawCards::vftable, frees the buffer at this+0x3c (its begin/end/cap at +0x3c/+0x40/+0x44 are reset to 0), frees the buffer… |
| `01424c40` | GameCommand_DrawCards::TypeName | confirmed | Returns the literal class name string GameCommand_DrawCards (returns a pointer to it). |
| `01424c80` | GameCommand_DrawCards::~GameCommand_DrawCards | confirmed | Scalar deleting destructor: calls the destructor body 01424b90, then calls _free on the object only when bit 0 of the flag argument is set, and returns the obj… |
| `01424ca0` | GameCommand_DrawCards::Dump | confirmed | Dump (vtable slot [19]): appends the text Draw Cards Game Command, then the base dump 0141b240, then Source: and the +0x24 value, Card count: from (end-begin)/… |
| `01424e40` | GameCommand_DrawCards::GameCommand_DrawCards | confirmed | Default constructor: calls base constructor 0141b5d0, stores GameCommand_DrawCards::vftable, then zeroes the fields at +0x24, +0x2c..+0x34 and +0x3c..+0x44. |
| `01424ec0` | GameCommand_DrawCards::GameCommand_DrawCards | confirmed | Constructor with one argument: calls base constructor 0141b5d0, stores GameCommand_DrawCards::vftable, zeroes the same buffer fields as 01424e40, sets +0x24 to… |
| `01424f60` | GameCommand_DrawCards::Clone | confirmed | Clone (vtable slot [17]): if this+8 (mGame) is zero it asserts (GameCommandDrawCards.cpp line 0x4d, mGame). Allocates 0x48 bytes with 00d833a6; when non-null i… |
| `01424ff0` | GameCommand_DrawCards::CopyFieldsTo | confirmed | CopyFieldsTo (vtable slot [18]): dynamic_casts its argument from NetworkCommand to GameCommand_DrawCards (00d8d382); on failure asserts (GameCommandDrawCards.c… |
| `01424c50` | GameCommand_DrawCards::GetStaticDword | likely | Returns the global dword DAT_01cbe27c (no arguments). |
| `01424c60` | GameCommand_DrawCards::SetStaticDword | likely | Stores its one argument into the global dword DAT_01cbe27c (no return value). |
| `01425050` | GameCommand_DrawCards::Factory | likely | Allocates 0x48 bytes with operator new (00d833a6, pushed 0x48). If the allocation is non-null it constructs the object with 01424e40 and returns it, otherwise … |

**GameCommand_FatalError** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014298a0` | GameCommand_FatalError::GameCommand_FatalError | confirmed | Default constructor: stores GameCommand_FatalError::vftable, constructs the secondary base at this+0x24 through 013ef350 (PersistentBase constructor per its vt… |
| `01429940` | GameCommand_FatalError::TypeName | confirmed | Returns the literal GameCommand_FatalError (mov eax, literal; ret). |
| `01429980` | GameCommand_FatalError::GameCommand_FatalError | confirmed | Constructor with one argument (the game): same body as 014298a0 (vtable store, secondary base at +0x24 via 013ef350, zeroed +0x88/+0x8c/+0x90), then calls the … |
| `01429a10` | GameCommand_FatalError::DtorBody | confirmed | Destructor body: stores GameCommand_FatalError::vftable, frees the buffer at this+0x88 (zeroing it) when non-null, then calls 013ef450 (secondary base destruct… |
| `01429950` | GameCommand_FatalError::GetStaticDword | likely | Returns the global dword DAT_01cbe284 (no arguments). |
| `01429960` | GameCommand_FatalError::SetStaticDword | likely | Stores its argument into the global dword DAT_01cbe284 (no return value). |

**GameCommand_OutOfCards** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01424400` | GameCommand_OutOfCards::Ctor | confirmed | Default constructor: calls base init 0141b5d0 on this, stores GameCommand_OutOfCards::vftable at +0 and returns this. |
| `01424480` | GameCommand_OutOfCards::GetClassName | confirmed | Virtual slot 11 of GameCommand_OutOfCards::vftable: returns the address of the literal 'GameCommand_OutOfCards' (bytes b8 7c 74 8e 01, c3). |
| `014244c0` | GameCommand_OutOfCards::CtorWithArg | confirmed | Constructor with one argument: calls base init 0141b5d0 on this, stores GameCommand_OutOfCards::vftable at +0, then calls 0141b3d0 on this with the argument; r… |
| `01424530` | GameCommand_OutOfCards::CtorAltBase | confirmed | Alternate constructor: stores GameCommand_OutOfCards::vftable at +0, then calls base init 0141b310 on this (not 0141b5d0); no callers in this shard. |
| `01424580` | GameCommand_OutOfCards::CloneNew | confirmed | Virtual slot 17 of GameCommand_OutOfCards::vftable: when 00d833a6 returns a non-null block, constructs a new object with 014244c0(param_1[2]); then calls this … |
| `01424750` | GameCommand_OutOfCards::~GameCommand_OutOfCards | confirmed | Scalar deleting destructor: stores GameCommand_OutOfCards::vftable into the object, calls the base GameCommand destructor body 0141b310, then calls _free on th… |
| `014247b0` | GameCommand_OutOfCards::Dump | confirmed | Appends the 26-byte literal Out of Cards Game Command newline to the output std::string argument via 01241650 (append), then calls 0141b240 with the same argum… |
| `014246d0` | GameCommand_OutOfCards::Factory | likely | Allocates 0x24 bytes with operator new (00d833a6, pushed 0x24). If the allocation is non-null it constructs the object with 01424400 (the constructor that stor… |

**GameCommand_OutOfSync** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01429470` | GameCommand_OutOfSync::GameCommand_OutOfSync | confirmed | Default constructor (EH frame): calls base constructor 0141b5d0 with this, then stores GameCommand_OutOfSync::vftable at this+0 and returns this. |
| `014294f0` | GameCommand_OutOfSync::TypeName | confirmed | Returns the literal GameCommand_OutOfSync (mov eax, literal; ret). |
| `01429530` | GameCommand_OutOfSync::GameCommand_OutOfSync | confirmed | Constructor with one argument (the game pointer): stores GameCommand_OutOfSync::vftable, calls base constructor 0141b3d0 with the argument, then calls 013508b0… |
| `01429600` | GameCommand_OutOfSync::Clone | confirmed | Clone (vtable slot [17]): allocates 0x24 bytes with operator new (00d833a6, pushed 0x24); if the allocation is non-null it constructs the copy with 01429530 us… |
| `01429670` | GameCommand_OutOfSync::CopyFieldsTo | confirmed | CopyFieldsTo (vtable slot [18]): dynamic_casts the argument from NetworkCommand to GameCommand_OutOfSync (00d8d382); on failure asserts (GameCommandOutOfSync.c… |
| `01429810` | GameCommand_OutOfSync::~GameCommand_OutOfSync | confirmed | Scalar deleting destructor: stores GameCommand_OutOfSync::vftable, calls base destructor 0141b310, then calls _free on the object when bit 0 of the flag argume… |
| `01429870` | GameCommand_OutOfSync::Dump | confirmed | Dump (vtable slot [19]): appends the 25-byte text Out Of Sync Game Command newline through 01241650, then calls base dump 0141b240 with the same argument. |
| `01429500` | GameCommand_OutOfSync::GetStaticDword | likely | Returns the global dword DAT_01cbe280 (no arguments). |
| `01429510` | GameCommand_OutOfSync::SetStaticDword | likely | Stores its argument into the global dword DAT_01cbe280 (no return value). |
| `01429790` | GameCommand_OutOfSync::Factory | likely | Allocates 0x24 bytes with operator new (00d833a6, pushed 0x24) and, when non-null, constructs the object with 01429470 (default constructor), returning it; ret… |

**LobbyCommand_SetDraft** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014be760` | LobbyCommand_SetDraft::Write | confirmed | Calls vftable slot 2 (header write), writes trace labels through the no-op, calls base helper 013b2960, writes the ints at this+8 (tournamentID), this+0xc (rou… |

**LobbyCommand_SetDraftReady** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014be030` | LobbyCommand_SetDraftReady::Read | confirmed | Calls vftable slot 4 (header read), base helper 013b2910, then reads the ints at this+8, this+0x10 and this+0xc through 013c24d0, and calls vftable slot 5 (bod… |
| `014be0a0` | LobbyCommand_SetDraftReady::Write | confirmed | Calls vftable slot 2 (header write), writes trace labels through the no-op, calls base helper 013b2960, then writes the ints at this+8 (tournamentID), this+0x1… |

**LobbyCommand_SetDraftedCards** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014be310` | LobbyCommand_SetDraftedCards::Write | confirmed | Calls vftable slot 2 (header write), writes trace labels through the no-op, calls base helper 013b2960, writes the int at this+8 (tournamentID) through 013c23f… |
| `014be530` | LobbyCommand_SetDraftedCards::Read | confirmed | Calls vftable slot 4 (header read), base helper 013b2910, reads the int at this+8 through 013c24d0, reads the container at this+0xc through 0135d6e0, then call… |

**LobbyCommand_SetFlag** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014bdc10` | LobbyCommand_SetFlag::Read | confirmed | Calls vftable slot 4 (header read), base helper 013b2910, reads the int at this+8 through 013c24d0, then calls vftable slot 5 (body read). |
| `014bdc60` | LobbyCommand_SetFlag::Write | confirmed | Calls vftable slot 2 (header write), writes trace labels through the no-op, calls base helper 013b2960, writes the int at this+8 through 013c23f0, then calls v… |

**LobbyCommand_SetLeagueCards** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014bd610` | LobbyCommand_SetLeagueCards::Write | confirmed | Calls vftable slot 2 (header write), writes trace labels through the no-op, calls base helper 013b2960, writes the int at this+8 (leagueID) through 013c23f0, w… |
| `014bd8e0` | LobbyCommand_SetLeagueCards::Read | confirmed | Calls vftable slot 4 (header read), base helper 013b2910, reads the int at this+8 through 013c24d0, reads the container at this+0xc through 01343fd0, then call… |

**LobbyCommand_SetLeagueCollection** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014bcd50` | LobbyCommand_SetLeagueCollection::Write | confirmed | Calls vftable slot 2 (header write), writes trace labels through the no-op, calls base helper 013b2960, writes the int at this+8 (leagueID), the container at t… |

**LobbyCommand_SetLeagueInfo** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014bcba0` | LobbyCommand_SetLeagueInfo::Read | confirmed | Calls vftable slot 4 (header read), base helper 013b2910, reads ints at this+8, +0xc and +0x28 through 013c24d0, the containers at this+0x10 and +0x1c through … |

**LobbyCommand_SetTournament** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014bc220` | LobbyCommand_SetTournament::Read | confirmed | Calls vftable slot 4 (header read), base helper 013b2910, reads the ints at this+8, +0x10 and +0xc through 013c24d0, then reads the int at this+0x14 only when … |
| `014bc2a0` | LobbyCommand_SetTournament::Write | confirmed | Calls vftable slot 2 (header write), writes trace labels through the no-op, calls base helper 013b2960, then writes the ints at this+8 (tournamentID), +0x10 (r… |

**LobbyCommand_SetupDraft** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014bb9b0` | LobbyCommand_SetupDraft::Write | confirmed | Calls vftable slot 2 (header write), writes trace labels through the no-op, calls base helper 013b2960, writes the int at this+8 (tournamentID), calls 012fbec0… |
| `014bba50` | LobbyCommand_SetupDraft::DestroyMembers | confirmed | Stores LobbyCommand_SetupDraft::vftable, frees the buffer at this+0x20 (when non-null) and zeroes this+0x20..+0x28, frees the buffer at this+0x10 and zeroes th… |
| `014bbb00` | LobbyCommand_SetupDraft::TypeName | confirmed | Returns the address of the literal 'LobbyCommand_SetupDraft'. |
| `014bbf60` | LobbyCommand_SetupDraft::Read | confirmed | Calls vftable slot 4 (header read), base helper 013b2910, reads the int at this+8 through 013c24d0, reads the sub-containers at this+0xc and this+0x1c through … |

**LobbyCommand_StartTournamentMsg** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014bb310` | LobbyCommand_StartTournamentMsg::Read | confirmed | Calls vftable slot 4 (header read), then base-level helper 013b2910, then reads the int at this+8 and the int at this+0xc through 013c24d0, then calls vftable … |
| `014bb370` | LobbyCommand_StartTournamentMsg::Write | confirmed | Calls vftable slot 2 (header write), writes the start/end trace labels through the no-op 013c26e0, calls base-level helper 013b2960, then writes the ints at th… |

**LobbyCommand_StartTournamentRound** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014bb780` | LobbyCommand_StartTournamentRound::Read | confirmed | Calls vftable slot 4 (header read), base-level helper 013b2910, then reads the ints at this+8 and this+0xc through 013c24d0, then calls vftable slot 5 (body re… |
| `014bb7e0` | LobbyCommand_StartTournamentRound::Write | confirmed | Calls vftable slot 2 (header write), writes trace labels 'tournamentID ' and 'round' through the no-op, calls base helper 013b2960, writes the ints at this+8 a… |

**LobbyCommand_UpdateTournament** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ba740` | LobbyCommand_UpdateTournament::Write | confirmed | Calls vftable slot 2 (header write), then writes trace labels (no-op) and: int at this+0x1c (GroupID), int at this+8 (TournamentID), container at this+0xc thro… |
| `014bafe0` | LobbyCommand_UpdateTournament::Read | confirmed | Calls vftable slot 4 (header read), then reads the int at this+0x1c, the int at this+8, the container at this+0xc through 01401800, the int at +0x20 and the in… |

**PilePlayArea** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0139d460` | PilePlayArea::TypeName | confirmed | Returns the literal string 'PilePlayArea'. |
| `0139d470` | PilePlayArea::StaticDwordGet | likely | Returns the global dword DAT_01cbdbdc. |
| `0139d480` | PilePlayArea::StaticDwordSet | likely | Stores its stack argument into DAT_01cbdbdc. |
| `0139e000` | PilePlayArea::DeletingDtorThunk_0x8 | likely | Thunk: subtracts 8 from ECX (the secondary subobject) and jumps to the PilePlayArea scalar deleting destructor 0139e190. |
| `0139e170` | PilePlayArea::RegisterFactory | likely | Static initializer: calls 013ce130 with classID 0xad and the factory 0139e0f0, then 013ce230 with the same pair. |

**PlayArea** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0139d330` | PlayArea::TypeName | confirmed | Returns the literal string 'PlayArea'. |
| `0139d340` | PlayArea::StaticDwordGet | likely | Returns the global dword DAT_01cbdbd8. |
| `0139d350` | PlayArea::StaticDwordSet | likely | Stores its stack argument into DAT_01cbdbd8. |
| `0139e020` | PlayArea::DeletingDtorThunk_0x8 | likely | Thunk: subtracts 8 from ECX and jumps to the PlayArea scalar deleting destructor 0139e0d0. |
| `0139e0b0` | PlayArea::RegisterFactory | likely | Static initializer: calls 013ce130 with classID 0xac and the factory 0139e030, then 013ce230 with the same classID and factory. |

**PlayerPlayArea** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0139d580` | PlayerPlayArea::TypeName | confirmed | Returns the literal string 'PlayerPlayArea'. |
| `0139d590` | PlayerPlayArea::StaticDwordGet | likely | Returns the global dword DAT_01cbdbe0. |
| `0139d5a0` | PlayerPlayArea::StaticDwordSet | likely | Stores its stack argument into DAT_01cbdbe0. |
| `0139e010` | PlayerPlayArea::DeletingDtorThunk_0x8 | likely | Thunk: subtracts 8 from ECX and jumps to the PlayerPlayArea scalar deleting destructor 0139e250. |
| `0139e230` | PlayerPlayArea::RegisterFactory | likely | Static initializer: calls 013ce130 with classID 0xae and the factory 0139e1b0, then 013ce230 with the same pair. |

**RequestTCGChallengePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a9ec0` | RequestTCGChallengePacket::RequestTCGChallengePacket 🌐 | confirmed | Constructor: calls the minigame base constructor FUN_008a4420 with sub-opcode 0x26 and param_2 in the third header word, then installs RequestTCGChallengePacke… |
| `009ac7b0` | DeletingDtor_009ac7b0 | likely | Scalar deleting destructor for a BasePacket-derived minigame packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object … |

**SynchronizationCommand_RequestAttachHolocron** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013a9bc0` | SynchronizationCommand_RequestAttachHolocron::DtorBody | confirmed | Stores SynchronizationCommand_RequestAttachHolocron::vftable into *this, sets its SEH state, and calls the base dtor body 01435430 on this (ECX). It frees noth… |
| `013a9d30` | SynchronizationCommand_RequestAttachHolocron::Deserialize | confirmed | Reads the stream arg1: this->vtbl[4](stream, &local) (class header, see 013aee40), then base 01435520 (which reads the header again), then two ints into this+4… |
| `013a9d90` | SynchronizationCommand_RequestAttachHolocron::Serialize | confirmed | Writes to the stream arg1: this->vtbl[2](stream) (class header, see 013aede0), trace text via 013c26e0 (a no-op), base 01435560 with the stream, then two ints … |
| `013a9e70` | SynchronizationCommand_RequestAttachHolocron::ScalarDeletin… | confirmed | Stores the class vftable into *this, calls the base dtor body 01435430 on this, and calls _free(this) when bit 0 of its byte argument is set. Returns this. |
| `013a9df0` | SynchronizationCommand_RequestAttachHolocron::Create | likely | Allocates 0xc bytes with operator new (00d833a6); if the result is null it returns 0; otherwise it constructs the object with 013a9b00 and returns it. |

**SynchronizationCommand_Update** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013d2f80` | SynchronizationCommand_Update::SynchronizationCommand_Update | confirmed | Constructor: calls 01435370 and stores SynchronizationCommand_Update::vftable; returns this. |
| `013d3000` | SynchronizationCommand_Update::CommandName | confirmed | Virtual slot 11: returns the address of the literal SynchronizationCommand_Update. |
| `013d3040` | SynchronizationCommand_Update::~SynchronizationCommand_Upda… | confirmed | Destructor body: stores the vtable, saves this in a local for the unwind frame and calls 01435430; does not free memory. |
| `013d30c0` | SynchronizationCommand_Update::Clone | confirmed | Virtual slot 17: allocates 8 bytes with 00d833a6, constructs a SynchronizationCommand_Update (013d2f80) in them, then calls slot 18 of this with the new object… |
| `013d3250` | SynchronizationCommand_Update::ScalarDeletingDtor | confirmed | Scalar deleting destructor with the body inlined: stores the vtable, calls 01435430, then calls _free on this when bit 0 of the second argument is set; returns… |
| `013d32b0` | SynchronizationCommand_Update::DebugDump | confirmed | Virtual slot 19: writes Update Synchronization Command through 01241650, then calls thunk 01435510 (which jumps to 0140c290) on the argument. |
| `013d32e0` | SynchronizationCommand_Update::EmitDebugName | confirmed | Virtual slot 20: passes the literal Update Synchronization Command (length 0x1e) to 01241650. |
| `013d3140` | SynchronizationCommand_Update::Deserialize | likely | Virtual slot 9: reads a header through slot 4, calls 01435520 with the argument, then calls slot 5 (+0x14); returns 0 on any failed step. |
| `013d3180` | SynchronizationCommand_Update::Serialize | likely | Virtual slot 10: calls slot 2 with its argument, writes trace text through 013c26e0 (a no-op), calls 01435560, writes the closing trace text through 013c26e0, … |
| `013d31d0` | SynchronizationCommand_Update::Create | likely | Allocates 8 bytes with 00d833a6; when non-null constructs a SynchronizationCommand_Update with 013d2f80 and returns it, otherwise returns 0. |

**TradingCardStartGamePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a9de0` | TradingCardStartGamePacket::TradingCardStartGamePacket 🌐 | confirmed | Constructor: calls the minigame base constructor FUN_008a4420 with sub-opcode 0x28, placing param_2 and param_5 in the family header, installs TradingCardStart… |
| `009ac750` | DeletingDtor_009ac750 | likely | Scalar deleting destructor for a BasePacket-derived minigame packet: rewrites *this to BasePacket::vftable and, when bit 0 of param_2 is set, frees the object … |

**(no class)** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009a9e70` | PacketDtor_009a9e70 | likely | Destructor stub: rewrites *this to BasePacket::vftable and returns; no member destruction and no delete. Called from the packet constructor path 009bf890 and a… |
| `009a9f30` | PacketDtor_009a9f30 | likely | Destructor stub: rewrites *this to BasePacket::vftable and returns; no member destruction and no delete. Called from an MSVC unwind funclet. |
| `0139b090` | CardReadIntMapEntries | likely | Reads an element count through 013c2690 (SerialStartReadContainer); for each element reads a value via 014019a0 from the reader at *param_1 and inserts it thro… |
| `013a9e50` | RegisterFactoryId0x23b | likely | Gets the registry object through 013ce130 (its EAX is moved to ECX) and inserts factory 013a9df0 under key 0x23b with 013ce230. |
| `01424730` | CommandFactoryRegister_0x45 | likely | Pushes the factory 014246d0 and the classID 0x45 and calls 013ce130, then passes its result in ECX to 013ce230. This registers the factory for classID 0x45 in … |
| `014250b0` | CommandFactoryRegister_0x44 | likely | Pushes the factory 01425050 and the classID 0x44 and calls 013ce130, then passes its result in ECX to 013ce230. This registers the factory for classID 0x44 in … |
| `014297f0` | CommandFactoryRegister_0x40 | likely | Pushes the factory 01429790 and the classID 0x40 and calls 013ce130, then passes its result in ECX to 013ce230. This registers the factory for classID 0x40 in … |

### `ui.deck`

**Engine::DeckAddFilter** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01273520` | Engine::DeckAddFilter::ActionName | confirmed | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string AddFilter. |
| `01273540` | Engine::DeckAddFilter::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string AddFilter : 0 : ..\common\...\deckhandlers.cc:242. |
| `01273550` | Engine::DeckAddFilter::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `01275320` | Engine::DeckAddFilter::Invoke | confirmed | Slot [2] of Engine::DeckAddFilter::vftable, 607 bytes. Reads PropertyKey and Comparison (FUN_012eb290) and a filter type dword; switch on the type: 0 reads an … |

**Engine::DeckCloseDeckBuilder** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01273130` | Engine::DeckCloseDeckBuilder::ActionName | confirmed | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string CloseDeckBuilder. |
| `01273150` | Engine::DeckCloseDeckBuilder::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string CloseDeckBuilder : 0 : ..\common\...\deckhandlers.cc:199. |
| `01273160` | Engine::DeckCloseDeckBuilder::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |

**Engine::DeckCollectionReadyDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01274560` | Engine::DeckCollectionReadyDisplayAction::Process | confirmed | Slot [2] of Engine::DeckCollectionReadyDisplayAction::vftable, 165 bytes. Assigns the 24-character string DataFinishedInitializing into a local std::string, pa… |

**Engine::DeckCollectionView** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01443170` | Engine::DeckCollectionView::DeckCollectionView | confirmed | Fastcall constructor (this in ECX, returns this). Calls the base constructor 01516200 on this, then stores the Engine::DeckCollectionView vtable at +0. |
| `014431d0` | Engine::DeckCollectionView::~DeckCollectionView | confirmed | Destructor body (fastcall, this in ECX, no deleting part). Stores Engine::DeckCollectionView::vftable at +0, calls the base destructor 01516260 on this and ret… |
| `01443220` | Engine::DeckCollectionView::ScalarDeletingDestructor | confirmed | Scalar deleting destructor (thiscall, one stack argument, ret 4). Stores Engine::DeckCollectionView::vftable at +0, calls the base destructor 01516260 on this,… |

**Engine::DeckDeckSelected** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01274250` | Engine::DeckDeckSelected::ActionName | confirmed | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string DeckSelected. |
| `01274270` | Engine::DeckDeckSelected::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string DeckSelected : 0 : ..\common\...\deckhandlers.cc:509. |
| `01274280` | Engine::DeckDeckSelected::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `01276190` | Engine::DeckDeckSelected::Invoke | confirmed | Slot [2] of Engine::DeckDeckSelected::vftable, 230 bytes. Gets the thread-owned object via FUN_0126dad0; reads the named field DAT_01811404 as a string (FUN_01… |

**Engine::DeckDeleteDeck** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01273e60` | Engine::DeckDeleteDeck::ActionName | confirmed | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string DeleteDeck. |
| `01273e80` | Engine::DeckDeleteDeck::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string DeleteDeck : 0 : ..\common\...\deckhandlers.cc:457. |
| `01273e90` | Engine::DeckDeleteDeck::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |

**Engine::DeckDoDeckSort** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01273910` | Engine::DeckDoDeckSort::ActionName | confirmed | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string DoDeckSort. |
| `01273930` | Engine::DeckDoDeckSort::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string DoDeckSort : 0 : ..\common\...\deckhandlers.cc:370. |
| `01273940` | Engine::DeckDoDeckSort::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `012758b0` | Engine::DeckDoDeckSort::Invoke | confirmed | Slot [2] of Engine::DeckDoDeckSort::vftable, 366 bytes. Reads an integer sort mode (FUN_012eb290 on DAT_017ea910) and calls slot 0x94 of the thread-owned objec… |

**Engine::DeckLoadDeck** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012743f0` | Engine::DeckLoadDeck::ctor | confirmed | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::DeckLoadDeck::vftable into the object; when the int argument is nonzero it ca… |
| `01274460` | Engine::DeckLoadDeck::ActionName | confirmed | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string LoadDeck. |
| `01274480` | Engine::DeckLoadDeck::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string LoadDeck : 0 : ..\common\...\deckhandlers.cc:63. |
| `01274490` | Engine::DeckLoadDeck::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |

**Engine::DeckOpenDeckBuilder** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01272f70` | Engine::DeckOpenDeckBuilder::ctor | confirmed | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::DeckOpenDeckBuilder::vftable into the object; when the int argument is nonzer… |
| `01272fe0` | Engine::DeckOpenDeckBuilder::ActionName | confirmed | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string OpenDeckBuilder. |
| `01273000` | Engine::DeckOpenDeckBuilder::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string OpenDeckBuilder : 0 : ..\common\...\deckhandlers.cc:163. |
| `01273010` | Engine::DeckOpenDeckBuilder::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `01274e30` | Engine::DeckOpenDeckBuilder::Invoke | likely | Slot [2] of Engine::DeckOpenDeckBuilder::vftable, 608 bytes. Gets the thread-owned object via FUN_0126dad0 and calls its slot 0x0c; forwards the named fields a… |

**Engine::DeckQueryDeck** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01273fb0` | Engine::DeckQueryDeck::ActionName | confirmed | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string QueryDeck. |
| `01273fd0` | Engine::DeckQueryDeck::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string QueryDeck : 0 : ..\common\...\deckhandlers.cc:478. |
| `01273fe0` | Engine::DeckQueryDeck::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `01275ec0` | Engine::DeckQueryDeck::Invoke | confirmed | Slot [2] of Engine::DeckQueryDeck::vftable, 414 bytes. Reads the named field DAT_01811404 as a string, creates an action named QueryDeck through FUN_012ec220, … |

**Engine::DeckQueryDecks** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01274100` | Engine::DeckQueryDecks::ActionName | confirmed | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string QueryDecks. |
| `01274120` | Engine::DeckQueryDecks::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string QueryDecks : 0 : ..\common\...\deckhandlers.cc:495. |
| `01274130` | Engine::DeckQueryDecks::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `01276070` | Engine::DeckQueryDecks::Invoke | confirmed | Slot [2] of Engine::DeckQueryDecks::vftable, 269 bytes. Creates an action named QueryDecks through FUN_012ec220; calls slot 0x38 of the thread-owned object for… |

**Engine::DeckQueryNewDeck** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01272e90` | Engine::DeckQueryNewDeck::ActionName | confirmed | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string QueryNewDeck. |
| `01272eb0` | Engine::DeckQueryNewDeck::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string QueryNewDeck : 0 : ..\common\...\deckhandlers.cc:125. |
| `01272ec0` | Engine::DeckQueryNewDeck::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |

**Engine::DeckQueryOpenDeck** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01273280` | Engine::DeckQueryOpenDeck::ActionName | confirmed | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string QueryOpenDeck. |
| `012732a0` | Engine::DeckQueryOpenDeck::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string QueryOpenDeck : 0 : ..\common\...\deckhandlers.cc:210. |
| `012732b0` | Engine::DeckQueryOpenDeck::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `012750a0` | Engine::DeckQueryOpenDeck::Invoke | confirmed | Slot [2] of Engine::DeckQueryOpenDeck::vftable, 303 bytes. Creates an action named OpenDialog through FUN_012ec220; writes ScreenName=LoadDeleteDeckDialog and … |

**Engine::DeckQuerySaveDeck** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01273360` | Engine::DeckQuerySaveDeck::ctor | confirmed | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::DeckQuerySaveDeck::vftable into the object; when the int argument is nonzero … |
| `012733d0` | Engine::DeckQuerySaveDeck::ActionName | confirmed | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string QuerySaveDeck. |
| `012733f0` | Engine::DeckQuerySaveDeck::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string QuerySaveDeck : 0 : ..\common\...\deckhandlers.cc:226. |
| `01273400` | Engine::DeckQuerySaveDeck::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `012751e0` | Engine::DeckQuerySaveDeck::Invoke | confirmed | Slot [2] of Engine::DeckQuerySaveDeck::vftable, 303 bytes. Creates an action named OpenDialog through FUN_012ec220; writes ScreenName=SaveDeckDialog and Dialog… |

**Engine::DeckQuerySelectDeck** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01273b50` | Engine::DeckQuerySelectDeck::ctor | confirmed | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::DeckQuerySelectDeck::vftable into the object; when the int argument is nonzer… |
| `01273bc0` | Engine::DeckQuerySelectDeck::ActionName | confirmed | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string QuerySelectDeck. |
| `01273be0` | Engine::DeckQuerySelectDeck::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string QuerySelectDeck : 0 : ..\common\...\deckhandlers.cc:423. |
| `01273bf0` | Engine::DeckQuerySelectDeck::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `01275a40` | Engine::DeckQuerySelectDeck::Invoke | confirmed | Slot [2] of Engine::DeckQuerySelectDeck::vftable, 456 bytes. Creates an action named OpenDialog through FUN_012ec220; writes ScreenName=SelectDeckDialog and Di… |

**Engine::DeckReloadCollectionDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012744f0` | Engine::DeckReloadCollectionDisplayAction::Process | likely | Slot [2] of Engine::DeckReloadCollectionDisplayAction::vftable, 30 bytes (decoded: call FUN_012e0ba0; call FUN_0126dad0; if non-null call its vtable slot at +0… |

**Engine::DeckReloadDeckDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01274510` | Engine::DeckReloadDeckDisplayAction::Process | likely | Slot [2] of Engine::DeckReloadDeckDisplayAction::vftable, 30 bytes, identical to 012744f0 (call FUN_012e0ba0; call FUN_0126dad0; if non-null call its vtable sl… |

**Engine::DeckRemoveFilter** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01273600` | Engine::DeckRemoveFilter::ctor | confirmed | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::DeckRemoveFilter::vftable into the object; when the int argument is nonzero i… |
| `01273670` | Engine::DeckRemoveFilter::ActionName | confirmed | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string RemoveFilter. |
| `01273690` | Engine::DeckRemoveFilter::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string RemoveFilter : 0 : ..\common\...\deckhandlers.cc:302. |
| `012736a0` | Engine::DeckRemoveFilter::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `012755a0` | Engine::DeckRemoveFilter::Invoke | confirmed | Slot [2] of Engine::DeckRemoveFilter::vftable, 531 bytes. Reads PropertyKey, Comparison and the filter type; type 0 reads an int Value and calls slot 0x7c (val… |

**Engine::DeckRequestDeckCollection** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01273d10` | Engine::DeckRequestDeckCollection::ActionName | confirmed | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string RequestDeckCollection. |
| `01273d30` | Engine::DeckRequestDeckCollection::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string RequestDeckCollection : 0 : ..\common\...\deckhandlers.cc:443. |
| `01273d40` | Engine::DeckRequestDeckCollection::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `01275c20` | Engine::DeckRequestDeckCollection::Invoke | confirmed | Slot [2] of Engine::DeckRequestDeckCollection::vftable, 202 bytes. Gets the thread-owned object via FUN_0126dad0, creates an action named InitializeDeckCollect… |

**Engine::DeckResetFilter** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012737c0` | Engine::DeckResetFilter::ActionName | confirmed | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string ResetFilters. |
| `012737e0` | Engine::DeckResetFilter::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string ResetFilters : 0 : ..\common\...\deckhandlers.cc:356. |
| `012737f0` | Engine::DeckResetFilter::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `012757c0` | Engine::DeckResetFilter::Invoke | likely | Slot [2] of Engine::DeckResetFilter::vftable, 234 bytes. Gets the thread-owned object; reads the Value field as a flag (FUN_012eb310) and passes it to virtual … |

**Engine::DeckSaveDeck** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01272d40` | Engine::DeckSaveDeck::ActionName | confirmed | Slot [1] of Engine::DeckSaveDeck::vftable. Bytes b8 imm32 c3: returns the constant C string SaveDeck. |
| `01272d60` | Engine::DeckSaveDeck::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string SaveDeck : 0 : ..\common\...\deckhandlers.cc:89. |
| `01272d70` | Engine::DeckSaveDeck::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `012748a0` | Engine::DeckSaveDeck::Invoke | likely | Slot [2] of Engine::DeckSaveDeck::vftable, 721 bytes. Gets the thread-owned object via FUN_0126dad0; if non-null: when the named field at DAT_01811404 exists p… |

**Engine::DeckSetSearch** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012739f0` | Engine::DeckSetSearch::ctor | confirmed | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::DeckSetSearch::vftable into the object; when the int argument is nonzero it c… |
| `01273a60` | Engine::DeckSetSearch::ActionName | confirmed | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string SetSearch. |
| `01273a80` | Engine::DeckSetSearch::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string SetSearch : 0 : ..\common\...\deckhandlers.cc:418. |
| `01273a90` | Engine::DeckSetSearch::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |

**Engine::DeckValidator** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01515b40` | Engine::DeckValidator::~DeckValidator | confirmed | Stores Engine::DeckValidator::vftable into *this, calls FUN_01537ec0 (base destructor), and returns; called from EH unwind funclets. |
| `01515b90` | Engine::DeckValidator::ScalarDeletingDtor | confirmed | Virtual slot 0 of Engine::DeckValidator. Stores the vtable, calls FUN_01537ec0, then calls _free on this when bit 0 of the second argument is set. Returns this. |
| `01515ae0` | Engine::DeckValidator::DeckValidator | likely | Calls FUN_01537e60 on this, then stores Engine::DeckValidator::vftable into *this and returns this. |

**Engine::DeckValidatorBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014393d0` | Engine::DeckValidatorBase::DeckValidatorBase | confirmed | Constructor (ECX = object). Stores Engine::DeckValidatorBase::vftable at +0, zeroes the dword at +4 and the byte at +8, sets the string capacity at +0x24 to 0x… |
| `01439430` | Engine::DeckValidatorBase::~DeckValidatorBase | confirmed | Destructor body (ECX = object). Stores Engine::DeckValidatorBase::vftable at +0, frees the string buffer at +0x10 with _free when the capacity at +0x24 is grea… |
| `014394a0` | Engine::DeckValidatorBase::ScalarDeletingDestructor | confirmed | Thiscall with one byte flag argument. Calls the destructor body 01439430 on the object, calls _free on the object when bit 0 of the flag is set, and returns th… |

**Engine::DeckView** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01443280` | Engine::DeckView::DeckView | confirmed | Fastcall constructor (this in ECX, returns this). Calls 0143f540 on this (unlabelled base constructor), then stores the Engine::DeckView vtable at +0. |
| `014432e0` | Engine::DeckView::~DeckView | confirmed | Destructor body (fastcall, this in ECX, no deleting part). Stores Engine::DeckView::vftable at +0, calls the base destructor 0143f5a0 on this and returns. |
| `01443330` | Engine::DeckView::ScalarDeletingDestructor | confirmed | Scalar deleting destructor (thiscall, one stack argument, ret 4). Stores Engine::DeckView::vftable at +0, calls the base destructor 0143f5a0 on this, then call… |

**Engine::DeckViewBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014395d0` | Engine::DeckViewBase::DeckViewBase | confirmed | Constructor (ECX = object, returns this). Stores Engine::DeckViewBase::vftable at +0, calls 0143b840 (the ArchetypeView constructor; receiver not decoded), the… |
| `01439640` | Engine::DeckViewBase::~DeckViewBase | confirmed | Destructor body (ECX = object). Stores Engine::DeckViewBase::vftable at +0, calls 01515b40 (not labelled), then 0143b8a0 (the ArchetypeView destructor body tha… |
| `014396e0` | Engine::DeckViewBase::ScalarDeletingDestructor | confirmed | Thiscall with one byte flag argument. Calls the destructor body 01439640 on the object, calls _free on the object when bit 0 of the flag is set, and returns th… |

**Engine::FRTWDeckAddCard** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0126e970` | Engine::FRTWDeckAddCard::Invoke | confirmed | Calls 012e0ba0, then 0126dad0 for the thread-owned object; when non-null adds the ArchetypeID key (012eb290) and a second key via 012eb290 with -1, builds an A… |

**Engine::FRTWDeckAddFilter** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012724a0` | Engine::FRTWDeckAddFilter::Invoke | confirmed | Slot [2] of Engine::FRTWDeckAddFilter::vftable, 820 bytes. Reads the filter type dword (FUN_012eb290) and, when the thread-owned object exists and the type is … |

**Engine::FRTWDeckAddHolocron** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0126f270` | Engine::FRTWDeckAddHolocron::Invoke | confirmed | Reads the HolocronID and CardID values (012eb290), looks up the card and holocron objects (012ae9a0, 012ae910, 012b8ff0, 012b8f80), and calls vtable slot 0x80 … |

**Engine::FRTWDeckCancelAttachHolocron** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0126de80` | Engine::FRTWDeckCancelAttachHolocron::Invoke | likely | Calls 012e0ba0, then 0126dad0; when the result is non-null calls its virtual slots 0xa4 and 0x9c with argument -1; returns 1. Virtual slot [2] of Engine::FRTWD… |

**Engine::FRTWDeckCollectionView** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01516200` | Engine::FRTWDeckCollectionView::Ctor | confirmed | Calls base constructor 014356b0 first, then stores the Engine::FRTWDeckCollectionView vtable into *this and returns this. |
| `01516260` | Engine::FRTWDeckCollectionView::Dtor | confirmed | Stores the FRTW deck view vtable into *this, calls base destructor 01435720 and returns without freeing the object. |
| `015162c0` | Engine::FRTWDeckCollectionView::ScalarDeletingDtor | confirmed | Virtual slot 0: stores the FRTW vtable into *this, calls base destructor 01435720, frees the object with _free (00d82463) when bit 0 of the flag argument is se… |

**Engine::FRTWDeckLoadWindowView** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0126ee80` | Engine::FRTWDeckLoadWindowView::Invoke | confirmed | Looks up the ArchetypeID value (012eb290) and calls 012b8f80/012ae910/012b8ff0 to find an object; when found, builds an AddModelView command (012ec110) and cal… |

**Engine::FRTWDeckMoveCard** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0126c9e0` | Engine::FRTWDeckMoveCard::ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable and then Engine::FRTWDeckMoveCard::vftable into *this, zeroes the local SEH state slot, calls 012ea4f0(this)… |
| `0126ca50` | Engine::FRTWDeckMoveCard::ActionName | confirmed | Returns the address of the literal "MoveCard" (mov eax,imm32; ret). No arguments and no side effects. Virtual slot [1] of Engine::FRTWDeckMoveCard::vftable. |
| `0126ca70` | Engine::FRTWDeckMoveCard::DebugName | confirmed | Returns the address of a literal beginning 'MoveCard : 0 : .' and ending 'frtwdeckhandlers.cc:41' (mov eax,imm32; ret). Virtual slot [4] of Engine::FRTWDeckMov… |
| `0126ca80` | Engine::FRTWDeckMoveCard::~FRTWDeckMoveCard | confirmed | Scalar deleting destructor shape: stores Scaleform::IScaleformActionHandler::vftable into *this, and when bit 0 of the byte flag argument is set calls _free(th… |
| `0126e780` | Engine::FRTWDeckMoveCard::Invoke | confirmed | Calls 012e0ba0, then 0126dad0 for the thread-owned object; when non-null it builds an AddCard command: keys ArchetypeID (012eb290), SourceSlot and TargetSlot (… |

**Engine::FRTWDeckRequestAttachHolocron** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0126dda0` | Engine::FRTWDeckRequestAttachHolocron::Invoke | likely | Calls 012e0ba0, then 0126dad0 for a thread-owned object; if non-null, calls its virtual slots at 0xa8, 0xa0, 0xa4 and 0x9c (0xa4 and 0x9c with argument -1), ca… |

**Engine::FRTWDeckRespondAttachHolocron** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0126cae0` | Engine::FRTWDeckRespondAttachHolocron::ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable and then Engine::FRTWDeckRespondAttachHolocron::vftable into *this, zeroes the local cookie slot, calls 012ea300(… |
| `0126cb70` | Engine::FRTWDeckRespondAttachHolocron::DebugName | confirmed | Returns the address of a literal beginning 'kRespondAttachHolocronDisplayAction : 0 : .' and ending in the deck handler source file and line 255 (mov eax,imm32… |
| `0126cb80` | Engine::FRTWDeckRespondAttachHolocron::~FRTWDeckRespondAtta… | confirmed | Scalar deleting destructor shape: stores Engine::IDisplayActionHandler::vftable into *this, and when bit 0 of the byte flag argument is set calls _free(this) (… |
| `0126f0f0` | Engine::FRTWDeckRespondAttachHolocron::Invoke | confirmed | Reads a value through 012f8f60(0); when its type field (+4) is 2 and the payload (+8) is non-zero it builds a RespondAttachHolocron command with Result 0 and R… |

**Engine::FRTWDeckSetFocus** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0126ed40` | Engine::FRTWDeckSetFocus::Invoke | confirmed | Calls 012e0ba0, then 0126dad0; when non-null reads the ArchetypeID value (012eb290), builds a SetFocus command with 012ec220, adds ArchetypeID and a Books entr… |

**Engine::FRTWDeckSetSpellbookColor** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0126dd80` | Engine::FRTWDeckSetSpellbookColor::Invoke | likely | Calls 012e0ba0, passes its result in ECX to 0126dad0 (thread-owned value lookup), and returns 1 (mov eax,1) with ret 8. Virtual slot [2] of Engine::FRTWDeckSet… |

**UI::DeckBuilderScreen** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012f7f90` | UI::DeckBuilderScreen::DeckBuilderScreen | confirmed | Constructor (fastcall, ECX = this): calls UI::FRTWDeckBuilderScreen::Ctor 013ff3b0 (which stores its own vtable), then stores UI::DeckBuilderScreen::vftable at… |
| `012f7ff0` | UI::DeckBuilderScreen::DtorBody | confirmed | Non-deleting destructor body: stores UI::DeckBuilderScreen::vftable at *this, then calls 013ff410 (UI::FRTWDeckBuilderScreen::Dtor). |
| `012f8040` | UI::DeckBuilderScreen::~DeckBuilderScreen | confirmed | Scalar deleting destructor (vtable slot [0]): stores UI::DeckBuilderScreen::vftable, calls the destructor body 013ff410, then calls _free on this when bit 0 of… |

**UI::DeckBuilderScreenBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01511bf0` | UI::DeckBuilderScreenBase::~DeckBuilderScreenBase | confirmed | Stores UI::DeckBuilderScreenBase::vftable into *this, then calls FUN_012f8480 (base destructor) and returns. |
| `01511c40` | UI::DeckBuilderScreenBase::ScalarDeletingDtor | confirmed | Virtual slot 0 of UI::DeckBuilderScreenBase. Stores the vtable, calls FUN_012f8480, then calls _free on this if flag bit 0 of the second argument is set. Retur… |
| `01511b90` | UI::DeckBuilderScreenBase::DeckBuilderScreenBase | likely | Calls FUN_012f8420 on this, then stores UI::DeckBuilderScreenBase::vftable into *this and returns this. |

**UI::FRTWDeckBuilderScreen** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013ff3b0` | UI::FRTWDeckBuilderScreen::Ctor | confirmed | Fastcall constructor: calls base constructor 01511b90, then stores UI::FRTWDeckBuilderScreen::vftable at this. |
| `013ff410` | UI::FRTWDeckBuilderScreen::Dtor | confirmed | Fastcall destructor body: stores UI::FRTWDeckBuilderScreen::vftable, then calls base destructor body 01511bf0. |
| `013ff480` | UI::FRTWDeckBuilderScreen::ScalarDeletingDtor | confirmed | Virtual slot 0 of UI::FRTWDeckBuilderScreen::vftable: stores the vtable, calls base destructor body 01511bf0, frees the object when the flag argument has bit 0… |

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01439990` | WriteDecksSection | likely | Thiscall-like helper with one argument (the output). Creates a temporary std::string via 012aab80 and 012aa280, walks a list of entries from begin to end, and … |

### `ui.tcg`

**Card** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01392710` | Card::FormatCardTitleTag | confirmed | Card virtual slot 66, writes '<cardtitle cardid="card\|%d">%s</cardtitle>' using card id this+0x34 and the name string at this+0x128 into the output string via… |
| `01391cf0` | Card::FormatTitleText | likely | Card virtual slot 86, writes the card title text into its string argument. If this+0x64 is zero it assigns '(unloaded card)'. Otherwise it calls this->vtable[1… |
| `013923e0` | Card::FormatCardTitleExtended | likely | Card virtual slot 67, writes the card reference markup into its string argument. First assigns the card name at this+0x128 to the argument. When this+0x64 is n… |

**ClientTCGInterface** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ad1640` | ClientTCGInterface::Ctor | confirmed | Stores the TCGInterface vtable, then replaces it with ClientTCGInterface's vtable. |
| `00ad16e0` | GetGlobalQword3c0 | confirmed | Returns the 64-bit field at +0x3c0 of the global client's object at +0x3e0. |
| `00ad1700` | RemoveEmitterForId | confirmed | Obtains the global client context and calls 00b08070 with the supplied id and zero. |
| `00ad1760` | ForwardGlobalObjectTo790d00 | confirmed | Calls the global-client getter, loads ECX from its +4 field, and tail-jumps to 00790d00. |
| `00ad1770` | ForwardGlobalObjectTo790d10 | confirmed | Calls the global-client getter, loads ECX from its +4 field, and tail-jumps to 00790d10. |
| `00ad1780` | GetGlobalByte940 | confirmed | Returns the byte at +0x940 of the global client object at +4. |
| `00ad1930` | GetGlobalField41c | confirmed | Returns the dword at +0x41c of the global client's object at +0x3e0. |
| `00ad1950` | CopyGlobalPair3a0 | confirmed | Copies the two dwords at global-client offsets +0x3a0 and +0x3a4 into the caller's output pair. |
| `00ad1970` | DispatchTwoGlobalHandlersA | confirmed | Calls 00769710 and then 00b4b820 with the same two supplied arguments. |
| `00ad19a0` | DispatchTwoGlobalHandlersB | confirmed | Calls 00769710 and then 00b4d210 with the same two supplied arguments. |
| `00ad19d0` | ClientTCGInterface::ScalarDeletingDtor | confirmed | Restores the TCGInterface base vtable and frees the object when flag bit 0 is set; otherwise returns it. |
| `00ad2790` | BuildT4DetailsText | confirmed | Builds a T4Details list from fixed input records, calls the integration API virtual slot +0x14 to resolve values, compacts the caller's text buffer, and return… |

**Engine::SystemMessageDisplayActionHandler** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0124f3c0` | Engine::SystemMessageDisplayActionHandler::SetTcgDialogTitle | confirmed | Slot [2] of Engine::SystemMessageDisplayActionHandler::vftable. Gets the object from FUN_00f9c010; if non-null it gets a ValueData from FUN_012f8f60(0), requir… |

**Engine::TournamentServiceBase** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013e48a0` | Engine::TournamentServiceBase::InvokeCurrentBracketInfo | confirmed | Virtual slot [49] of Engine::TournamentServiceBase (also slot [49] of Engine::TournamentService and Engine::FRTWTournamentService). If the group accessor FUN_0… |
| `013e4d80` | Engine::TournamentServiceBase::InvokeRequestBracketPageChan… | confirmed | Virtual slot [51]. Reads a value through this->vtable +0xe8 then +0xe4, creates Invoke action 'RequestBracketPageChange' (012ec220), sets integer parameter Gro… |
| `013e4e90` | Engine::TournamentServiceBase::InvokeRequestTournamentPageC… | confirmed | Virtual slot [52]. Creates Invoke action 'RequestTournamentPageChange' (012ec220), sets string parameter PageName (012ebe00) from its argument, then calls 012e… |
| `013e4f80` | Engine::TournamentServiceBase::InvokeBracketListenForGroup | confirmed | Virtual slot [53]. Takes an integer argument, creates Invoke action 'BracketListenForGroup' (012ec220), sets integer parameter GroupID (012ebe40) from the argu… |
| `013e6a20` | Engine::TournamentServiceBase::PushUpdatedEventLeaderboard | confirmed | Virtual slot [44] of Engine::TournamentServiceBase (also of TournamentService and FRTWTournamentService). Gets an iterator from the map at this+0x48 through 01… |

**GuiProperties<GuiTradingCardCtrl>** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010ce760` | GuiProperties_GuiTradingCardCtrl::Ctor | confirmed | Fastcall with ECX=this. Stores the global dword at 017efbd4 into this+4 and this+8, zeroes this+0xc and this+0x10, stores GuiProperties<GuiTradingCardCtrl>::vf… |
| `010ceb00` | GuiProperties_GuiTradingCardCtrl::DtorBody | confirmed | Stores the GuiProperties vftable at this, then the embedded SoeUtil::Map vftable at this+0x14. If the map root at this+0x18 is non-null it applies FUN_008234e0… |
| `010cf610` | GuiProperties_GuiTradingCardCtrl::CopyCtor | confirmed | Constructs a GuiProperties table at ECX=this from a source table param_2: stores the global dword 017efbd4 into this+4 and this+8, zeroes this+0xc and this+0x1… |
| `010cecd0` | GuiProperties_GuiTradingCardCtrl::ScalarDeletingDtor | likely | Virtual method at slot [28] of GuiProperties<GuiTradingCardCtrl>::vftable with the deleting-destructor shape: calls the body destructor 010ceb00 on this, then … |
| `010cfa40` | GuiPropertiesFormatSHashOrDelegate | likely | Virtual method at slot [9] of GuiProperties<GuiTradingCardCtrl>::vftable (also GuiTradingCardCtrl +0x158[9]). Looks up the static name pointer at DAT_01b34f8c[… |

**GuiTradingCardCtrl** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010cee90` | GuiTradingCardCtrl::Ctor | confirmed | Fastcall with ECX=this. Calls 00864400 on this, then constructs the GuiProperties table at this+0x158 with 010ce760 (ECX=this+0x158). Stores 018876cc at this+0… |
| `010cf020` | GuiTradingCardCtrl::DtorBody | confirmed | Stores the GuiTradingCardCtrl vftables at this, this+4, this+0x24, this+0xe8 and this+0x158 (the last one via the same vftable as the GuiProperties sub-object)… |
| `010cf310` | GuiTradingCardCtrl::ScalarDeletingDtor | confirmed | Slot [0] of GuiTradingCardCtrl::vftable with the scalar deleting destructor shape: calls the body destructor 010cf020 on this, then if bit 0 of param_2 is set … |
| `010cfb20` | GuiTradingCardCtrl::CopyCtor | confirmed | Fastcall copy constructor: ECX=this, one stack argument param_2 (source control). Calls 00845240 with the source (ECX stays this), then copy-constructs the emb… |
| `010cf2c0` | GuiTradingCardCtrl::CtorIfNonNull | likely | Stack argument param_1 is treated as the object address: when it is non-zero it calls 010cee90 (GuiTradingCardCtrl constructor) with ECX set to that address. R… |
| `010cfcb0` | GuiTradingCardCtrl::CopyCtorIfNonNull | likely | Stack argument param_1 is the destination and param_2 the source. When the destination is non-zero it calls 010cfb20 (GuiTradingCardCtrl copy constructor) with… |
| `010cfd10` | GuiTradingCardCtrlLifecycle | likely | Switch on the operation code param_1 (stack args param_2 and param_3, returns a pointer): 0 runs the constructor 010cee90 on param_2 when non-null and returns … |

**TCGInterface** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ad1170` | TCGInterface::Dtor | confirmed | Restores the TCGInterface vtable at the start of the object and returns. |
| `00ad1180` | TCGInterface::ScalarDeletingDtor | confirmed | Restores the TCGInterface vtable, frees the object when flag bit 0 is set, and otherwise returns the object pointer. |
| `00ad1790` | TCGInterface::Ctor | confirmed | Installs the TCGInterface vtable and returns. |

**UI::TouramentRequestJoinMatchingQueue** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123f8a0` | UI::TouramentRequestJoinMatchingQueue::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable then UI::TouramentRequestJoinMatchingQueue::vftable into the object; when param_2 is nonzero it calls 012ea4… |
| `0123f910` | UI::TouramentRequestJoinMatchingQueue::GetActionName | confirmed | Returns the address of the literal RequestJoinMatchingQueue (mov eax, imm32; ret). |
| `0123f930` | UI::TouramentRequestJoinMatchingQueue::GetTraceLabel | confirmed | Returns the address of the literal RequestJoinMatchingQueue : 0 : trace string (tournamenthandlers.cc:1058) (mov eax, imm32; ret). |
| `0123f940` | UI::TouramentRequestJoinMatchingQueue::ScalarDeletingDtor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the object, then calls _free on it when bit 0 of the flag argument is set; returns this. |
| `01241ed0` | UI::TouramentRequestJoinMatchingQueue::Vslot2 | likely | Virtual slot [2]: calls 012e0ba0, looks the object up through 01241840 and calls its vtable +0x74 with no arguments when non-null; returns 1. |

**UI::TournamentAddLobbyGroup** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01240710` | UI::TournamentAddLobbyGroup::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable then UI::TournamentAddLobbyGroup::vftable into the object; when param_2 is nonzero it calls 012ea300(this). |
| `012407a0` | UI::TournamentAddLobbyGroup::GetTraceLabel | confirmed | Returns the address of the literal kAddLobbyGroupDisplayAction : 0 : trace string (mov eax, imm32; ret). |
| `012407b0` | UI::TournamentAddLobbyGroup::ScalarDeletingDtor | confirmed | Stores Engine::IDisplayActionHandler::vftable into the object, then calls _free on it when bit 0 of the flag argument is set; returns this. |
| `0123c510` | UI::TournamentAddLobbyGroup::CheckThreeIntArgs | likely | Virtual slot [2] of UI::TournamentAddLobbyGroup::vftable: calls FUN_012f8f60 for argument indices 0, 1 and 2, asserts through FUN_012f5a60 that each value's ty… |

**UI::TournamentCacheEventLeaderboardScaleformAction** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123f4b0` | UI::TournamentCacheEventLeaderboardScaleformAction::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable then UI::TournamentCacheEventLeaderboardScaleformAction::vftable into the object; when param_2 is nonzero it… |
| `0123f520` | UI::TournamentCacheEventLeaderboardScaleformAction::GetActi… | confirmed | Returns the address of the literal CacheEventLeaderboards (mov eax, imm32; ret). |
| `0123f540` | UI::TournamentCacheEventLeaderboardScaleformAction::GetTrac… | confirmed | Returns the address of the literal CacheEventLeaderboards : 0 : tournamenthandlers.cc:1001 trace string (mov eax, imm32; ret). |
| `0123f550` | UI::TournamentCacheEventLeaderboardScaleformAction::ScalarD… | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the object, then calls _free on it when bit 0 of the flag argument is set; returns this. |
| `01247170` | UI::TournamentCacheEventLeaderboardScaleformAction::CallSlo… | likely | Virtual slot [2] of UI::TournamentCacheEventLeaderboardScaleformAction (vftable 018c98fc). Reads the int under the key Leaderboard (default 0) with 012eb290, p… |

**UI::TournamentCancelJoinMatchingQueue** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123f9f0` | UI::TournamentCancelJoinMatchingQueue::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable then UI::TournamentCancelJoinMatchingQueue::vftable into the object; when param_2 is nonzero it calls 012ea4… |
| `0123fa60` | UI::TournamentCancelJoinMatchingQueue::GetActionName | confirmed | Returns the address of the literal CancelJoinMatchingQueue (mov eax, imm32; ret). |
| `0123fa80` | UI::TournamentCancelJoinMatchingQueue::GetTraceLabel | confirmed | Returns the address of the literal CancelJoinMatchingQueue : 0 : trace string (tournamenthandlers.cc:1072) (mov eax, imm32; ret). |
| `0123fa90` | UI::TournamentCancelJoinMatchingQueue::ScalarDeletingDtor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the object, then calls _free on it when bit 0 of the flag argument is set; returns this. |
| `01241f00` | UI::TournamentCancelJoinMatchingQueue::Vslot2 | likely | Virtual slot [2]: calls 012e0ba0, looks the object up through 01241840 and calls its vtable +0x78 with no arguments when non-null; returns 1. |

**UI::TournamentDeckSelected** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123fb40` | UI::TournamentDeckSelected::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable then UI::TournamentDeckSelected::vftable into the object; when param_2 is nonzero it calls 012ea4f0(this). |
| `0123fbb0` | UI::TournamentDeckSelected::GetActionName | confirmed | Returns the address of the literal DeckSelected (mov eax, imm32; ret). |
| `0123fbd0` | UI::TournamentDeckSelected::GetTraceLabel | confirmed | Returns the address of the literal DeckSelected : 2 : trace string (tournamenthandlers.cc:1084) (mov eax, imm32; ret). |
| `0123fbe0` | UI::TournamentDeckSelected::ScalarDeletingDtor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the object, then calls _free on it when bit 0 of the flag argument is set; returns this. |
| `012444f0` | UI::TournamentDeckSelected::ReadIdAndForward | confirmed | Virtual slot [2] of UI::TournamentDeckSelected (vftable 018c9b88). ECX is not read; the object is the first stack argument. Reads string key "ID" via 012eb250 … |

**UI::TournamentError** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123d0b0` | UI::TournamentError::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentError::vftable into the object, and when the flag argument is nonzero calls FUN_012ea300 o… |
| `0123d150` | UI::TournamentError::ScalarDeletingDtor | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set, after storing the base vtable. |
| `0123d140` | UI::TournamentError::GetTraceLabel | likely | Virtual slot [4] of UI::TournamentError::vftable: returns the address of its diagnostic literal. |
| `01243450` | UI::TournamentError::Vslot2 | likely | Virtual slot [2]: starts a local std::string with the text [Error Getting String]; when argument 0 is a string (type tag 3, otherwise asserts through 012f5a60)… |

**UI::TournamentErrorCode** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123cf10` | UI::TournamentErrorCode::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentErrorCode::vftable into the object, and when the flag argument is nonzero calls FUN_012ea3… |
| `0123cfb0` | UI::TournamentErrorCode::ScalarDeletingDtor | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set, after storing the base vtable. |
| `0123cfa0` | UI::TournamentErrorCode::GetTraceLabel | likely | Virtual slot [4] of UI::TournamentErrorCode::vftable: returns the address of its diagnostic literal. |
| `0123d010` | UI::TournamentErrorCode::ValidateArg0Int | likely | Virtual slot [2] of UI::TournamentErrorCode::vftable: calls FUN_012f8f60(0) and, when its type id at +4 equals 2, rechecks argument 0 with the same test and as… |

**UI::TournamentEventCommandNotifyMatchEnded** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123f0c0` | UI::TournamentEventCommandNotifyMatchEnded::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable then UI::TournamentEventCommandNotifyMatchEnded::vftable into the object; when param_2 is nonzero it calls 012ea3… |
| `0123f150` | UI::TournamentEventCommandNotifyMatchEnded::GetTraceLabel | confirmed | Returns the address of the literal kEventCommandNotifyMatchEnded trace string (mov eax, imm32; ret, 6 bytes). |
| `0123f160` | UI::TournamentEventCommandNotifyMatchEnded::ScalarDeletingD… | confirmed | Stores Engine::IDisplayActionHandler::vftable into the object, then calls _free on it when bit 0 of the flag argument is set; returns this. |
| `01241bf0` | UI::TournamentEventCommandNotifyMatchEnded::Vslot2 | likely | Virtual slot [2]: reads arguments 0 and 1 as integers (type tag 2) through 012f8f60, asserting through 012f5a60 on mismatch; calls 012e0ba0, looks the object u… |

**UI::TournamentEventSendEventPlayerFlagsDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123ee20` | UI::TournamentEventSendEventPlayerFlagsDisplayAction::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentEventSendEventPlayerFlagsDisplayAction::vftable into the object, and when the flag argumen… |
| `0123eec0` | UI::TournamentEventSendEventPlayerFlagsDisplayAction::Scala… | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
| `0123eeb0` | UI::TournamentEventSendEventPlayerFlagsDisplayAction::GetTr… | likely | Virtual slot [4] of UI::TournamentEventSendEventPlayerFlagsDisplayAction::vftable: returns the address of its diagnostic literal. |

**UI::TournamentEventSendReadyStatusDisplayAction** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123ecd0` | UI::TournamentEventSendReadyStatusDisplayAction::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentEventSendReadyStatusDisplayAction::vftable into the object, and when the flag argument is … |
| `0123ed70` | UI::TournamentEventSendReadyStatusDisplayAction::ScalarDele… | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
| `0123ed60` | UI::TournamentEventSendReadyStatusDisplayAction::GetTraceLa… | likely | Virtual slot [4] of UI::TournamentEventSendReadyStatusDisplayAction::vftable: returns the address of its diagnostic literal. |
| `01241af0` | UI::TournamentEventSendReadyStatusDisplayAction::Vslot2 | likely | Virtual slot [2]: reads argument 0 as an integer (type tag 2) and argument 1 as a boolean (type tag 1) through 012f8f60, with asserts through 012f5a60 on misma… |

**UI::TournamentJoinTournamentLobby** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01240810` | UI::TournamentJoinTournamentLobby::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable then UI::TournamentJoinTournamentLobby::vftable into the object; when param_2 is nonzero it calls 012ea4f0(t… |
| `01240880` | UI::TournamentJoinTournamentLobby::GetActionName | confirmed | Returns the address of the literal JoinTournamentLobby (mov eax, imm32; ret). |
| `012408a0` | UI::TournamentJoinTournamentLobby::GetTraceLabel | confirmed | Returns the address of the literal JoinTournamentLobby : 0 : trace string (tournamenthandlers.cc:347) (mov eax, imm32; ret). |
| `012408b0` | UI::TournamentJoinTournamentLobby::ScalarDeletingDtor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the object, then calls _free on it when bit 0 of the flag argument is set; returns this. |
| `01243550` | UI::TournamentJoinTournamentLobby::Vslot2 | likely | Virtual slot [2]: when the local pointer (argument-derived) is nonzero builds the string JoinLobby and calls 012ec110 for a result; sets the named values Group… |

**UI::TournamentJoinTournamentSubLobby** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123d640` | UI::TournamentJoinTournamentSubLobby::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable and then UI::TournamentJoinTournamentSubLobby::vftable into the object, and when the flag argument is nonzer… |
| `0123d6e0` | UI::TournamentJoinTournamentSubLobby::ScalarDeletingDtor | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
| `01245e90` | UI::TournamentJoinTournamentSubLobby::QueueJoinLobby | confirmed | Virtual slot [2] of UI::TournamentJoinTournamentSubLobby (vftable 018c8ddc). Takes no arguments. Reads argument 5 as a list through 0130a060 and, for each elem… |
| `0123d6b0` | UI::TournamentJoinTournamentSubLobby::GetActionName | likely | Virtual slot [1] of UI::TournamentJoinTournamentSubLobby::vftable: returns the address of the literal JoinTournamentSubLobby. |
| `0123d6d0` | UI::TournamentJoinTournamentSubLobby::GetTraceLabel | likely | Virtual slot [4] of UI::TournamentJoinTournamentSubLobby::vftable: returns the address of its diagnostic literal. |

**UI::TournamentLeaderBoardDataUpdatedDisplayAction** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123e640` | UI::TournamentLeaderBoardDataUpdatedDisplayAction::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentLeaderBoardDataUpdatedDisplayAction::vftable into the object, and when the flag argument i… |
| `0123e6e0` | UI::TournamentLeaderBoardDataUpdatedDisplayAction::ScalarDe… | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
| `0123e6d0` | UI::TournamentLeaderBoardDataUpdatedDisplayAction::GetTrace… | likely | Virtual slot [4] of UI::TournamentLeaderBoardDataUpdatedDisplayAction::vftable: returns the address of its diagnostic literal. |
| `01241950` | UI::TournamentLeaderBoardDataUpdatedDisplayAction::Vslot2 | likely | Virtual slot [2]: calls 012e0ba0, then looks up the object through 01241840 and, when it is non-null, calls its vtable slot +0x48; returns 1. Takes no argument… |

**UI::TournamentLeaveTournamentSubLobby** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123d790` | UI::TournamentLeaveTournamentSubLobby::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable and then UI::TournamentLeaveTournamentSubLobby::vftable into the object, and when the flag argument is nonze… |
| `0123d830` | UI::TournamentLeaveTournamentSubLobby::ScalarDeletingDtor | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
| `01246040` | UI::TournamentLeaveTournamentSubLobby::QueueLeaveLobby | confirmed | Virtual slot [2] of UI::TournamentLeaveTournamentSubLobby (vftable 018c8e5c). Takes no arguments. Reads argument 5 as a list through 0130a060 and, for each ele… |
| `0123d800` | UI::TournamentLeaveTournamentSubLobby::GetActionName | likely | Virtual slot [1] of UI::TournamentLeaveTournamentSubLobby::vftable: returns the address of the literal LeaveTournamentSubLobby. |
| `0123d820` | UI::TournamentLeaveTournamentSubLobby::GetTraceLabel | likely | Virtual slot [4] of UI::TournamentLeaveTournamentSubLobby::vftable: returns the address of its diagnostic literal. |

**UI::TournamentOnJoinTournamentLobby** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123da30` | UI::TournamentOnJoinTournamentLobby::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable and then UI::TournamentOnJoinTournamentLobby::vftable into the object, and when the flag argument is nonzero… |
| `0123dad0` | UI::TournamentOnJoinTournamentLobby::ScalarDeletingDtor | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
| `0123daa0` | UI::TournamentOnJoinTournamentLobby::GetActionName | likely | Virtual slot [1] of UI::TournamentOnJoinTournamentLobby::vftable: returns the address of the literal OnJoinTournamentLobby. |
| `0123dac0` | UI::TournamentOnJoinTournamentLobby::GetTraceLabel | likely | Virtual slot [4] of UI::TournamentOnJoinTournamentLobby::vftable: returns the address of its diagnostic literal. |

**UI::TournamentOnJoinTournamentSubLobby** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123dfa0` | UI::TournamentOnJoinTournamentSubLobby::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable and then UI::TournamentOnJoinTournamentSubLobby::vftable into the object, and when the flag argument is nonz… |
| `0123e040` | UI::TournamentOnJoinTournamentSubLobby::ScalarDeletingDtor | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
| `0123e010` | UI::TournamentOnJoinTournamentSubLobby::GetActionName | likely | Virtual slot [1] of UI::TournamentOnJoinTournamentSubLobby::vftable: returns the address of the literal OnJoinTournamentSubLobby. |
| `0123e030` | UI::TournamentOnJoinTournamentSubLobby::GetTraceLabel | likely | Virtual slot [4] of UI::TournamentOnJoinTournamentSubLobby::vftable: returns the address of its diagnostic literal. |
| `01243a00` | UI::TournamentOnJoinTournamentSubLobby::Vslot2 | likely | Virtual slot [2] with no arguments: reads named values tournamentID and groupID through 012eb290, playerRole with type argument 2 (kIntegerTypeID) and password… |

**UI::TournamentOnLeaveTournamentSubLobby** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123e0f0` | UI::TournamentOnLeaveTournamentSubLobby::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable and then UI::TournamentOnLeaveTournamentSubLobby::vftable into the object, and when the flag argument is non… |
| `0123e190` | UI::TournamentOnLeaveTournamentSubLobby::ScalarDeletingDtor | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
| `0123e160` | UI::TournamentOnLeaveTournamentSubLobby::GetActionName | likely | Virtual slot [1] of UI::TournamentOnLeaveTournamentSubLobby::vftable: returns the address of the literal OnLeaveTournamentSubLobby. |
| `0123e180` | UI::TournamentOnLeaveTournamentSubLobby::GetTraceLabel | likely | Virtual slot [4] of UI::TournamentOnLeaveTournamentSubLobby::vftable: returns the address of its diagnostic literal. |

**UI::TournamentOnSelectTournamentDeck** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123de40` | UI::TournamentOnSelectTournamentDeck::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable and then UI::TournamentOnSelectTournamentDeck::vftable into the object, and when the flag argument is nonzer… |
| `0123dee0` | UI::TournamentOnSelectTournamentDeck::ScalarDeletingDtor | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
| `0123deb0` | UI::TournamentOnSelectTournamentDeck::GetActionName | likely | Virtual slot [1] of UI::TournamentOnSelectTournamentDeck::vftable: returns the address of the literal OnSelectTournamentDeck. |
| `0123ded0` | UI::TournamentOnSelectTournamentDeck::GetTraceLabel | likely | Virtual slot [4] of UI::TournamentOnSelectTournamentDeck::vftable: returns the address of its diagnostic literal. |

**UI::TournamentOnSelectTournamentJoin** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123dce0` | UI::TournamentOnSelectTournamentJoin::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable and then UI::TournamentOnSelectTournamentJoin::vftable into the object, and when the flag argument is nonzer… |
| `0123dd80` | UI::TournamentOnSelectTournamentJoin::ScalarDeletingDtor | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
| `0123dd50` | UI::TournamentOnSelectTournamentJoin::GetActionName | likely | Virtual slot [1] of UI::TournamentOnSelectTournamentJoin::vftable: returns the address of the literal OnSelectTournamentJoin. |
| `0123dd70` | UI::TournamentOnSelectTournamentJoin::GetTraceLabel | likely | Virtual slot [4] of UI::TournamentOnSelectTournamentJoin::vftable: returns the address of its diagnostic literal. |

**UI::TournamentOnShowRewardDetails** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123e3a0` | UI::TournamentOnShowRewardDetails::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable and then UI::TournamentOnShowRewardDetails::vftable into the object, and when the flag argument is nonzero c… |
| `0123e440` | UI::TournamentOnShowRewardDetails::ScalarDeletingDtor | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
| `0123e410` | UI::TournamentOnShowRewardDetails::GetActionName | likely | Virtual slot [1] of UI::TournamentOnShowRewardDetails::vftable: returns the address of the literal OnShowRewardDetails. |
| `0123e430` | UI::TournamentOnShowRewardDetails::GetTraceLabel | likely | Virtual slot [4] of UI::TournamentOnShowRewardDetails::vftable: returns the address of its diagnostic literal. |

**UI::TournamentOnShowTournamentDetails** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123e250` | UI::TournamentOnShowTournamentDetails::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable and then UI::TournamentOnShowTournamentDetails::vftable into the object, and when the flag argument is nonze… |
| `0123e2f0` | UI::TournamentOnShowTournamentDetails::ScalarDeletingDtor | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
| `0123e2c0` | UI::TournamentOnShowTournamentDetails::GetActionName | likely | Virtual slot [1] of UI::TournamentOnShowTournamentDetails::vftable: returns the address of the literal OnShowTournamentDetails. |
| `0123e2e0` | UI::TournamentOnShowTournamentDetails::GetTraceLabel | likely | Virtual slot [4] of UI::TournamentOnShowTournamentDetails::vftable: returns the address of its diagnostic literal. |

**UI::TournamentOnTournamentLobbyList** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123db90` | UI::TournamentOnTournamentLobbyList::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable and then UI::TournamentOnTournamentLobbyList::vftable into the object, and when the flag argument is nonzero… |
| `0123dc30` | UI::TournamentOnTournamentLobbyList::ScalarDeletingDtor | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
| `0123dc00` | UI::TournamentOnTournamentLobbyList::GetActionName | likely | Virtual slot [1] of UI::TournamentOnTournamentLobbyList::vftable: returns the address of the literal OnTournamentLobbyList. |
| `0123dc20` | UI::TournamentOnTournamentLobbyList::GetTraceLabel | likely | Virtual slot [4] of UI::TournamentOnTournamentLobbyList::vftable: returns the address of its diagnostic literal. |

**UI::TournamentOpenTournament** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123fde0` | UI::TournamentOpenTournament::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable then UI::TournamentOpenTournament::vftable into the object; when param_2 is nonzero it calls 012ea4f0(this). |
| `0123fe50` | UI::TournamentOpenTournament::GetActionName | confirmed | Returns the address of the literal OpenTournament (mov eax, imm32; ret). |
| `0123fe70` | UI::TournamentOpenTournament::GetTraceLabel | confirmed | Returns the address of the literal OpenTournament : 0 : trace string (tournamenthandlers.cc:1112) (mov eax, imm32; ret). |
| `0123fe80` | UI::TournamentOpenTournament::ScalarDeletingDtor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the object, then calls _free on it when bit 0 of the flag argument is set; returns this. |
| `01241f60` | UI::TournamentOpenTournament::Vslot2 | likely | Virtual slot [2] with one stack argument: calls 012e0ba0, looks the object up through 01241840 and calls its vtable +0x84 with that argument when non-null; ret… |

**UI::TournamentQueryEventTickets** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123fc90` | UI::TournamentQueryEventTickets::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable then UI::TournamentQueryEventTickets::vftable into the object; when param_2 is nonzero it calls 012ea4f0(thi… |
| `0123fd00` | UI::TournamentQueryEventTickets::GetActionName | confirmed | Returns the address of the literal QueryEventTickets (mov eax, imm32; ret). |
| `0123fd20` | UI::TournamentQueryEventTickets::GetTraceLabel | confirmed | Returns the address of the literal QueryEventTickets : 0 : trace string (tournamenthandlers.cc:1100) (mov eax, imm32; ret). |
| `0123fd30` | UI::TournamentQueryEventTickets::ScalarDeletingDtor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the object, then calls _free on it when bit 0 of the flag argument is set; returns this. |
| `01241f30` | UI::TournamentQueryEventTickets::Vslot2 | likely | Virtual slot [2]: calls 012e0ba0, looks the object up through 01241840 and calls its vtable +0x80 with no arguments when non-null; returns 1. |

**UI::TournamentReloadDeck** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123c9d0` | UI::TournamentReloadDeck::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentReloadDeck::vftable into the object, and when the flag argument is nonzero calls FUN_012ea… |
| `0123ca70` | UI::TournamentReloadDeck::ScalarDeletingDtor | confirmed | Stores Engine::IDisplayActionHandler::vftable into the object, then frees it through _free when bit 0 of the flag argument is set. |
| `0123ca60` | UI::TournamentReloadDeck::GetTraceLabel | likely | Virtual slot [4] of UI::TournamentReloadDeck::vftable: returns the address of the literal kReloadDeckDisplayAction label with its source file and line. |
| `012429b0` | UI::TournamentReloadDeck::Vslot2 | likely | Virtual slot [2] with no arguments: builds a named bundle from the literal ReloadDeck (0130df10), then when 00f9c010 is nonzero dispatches it through 012ebc00,… |

**UI::TournamentRequestSessionInfo** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123f750` | UI::TournamentRequestSessionInfo::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable then UI::TournamentRequestSessionInfo::vftable into the object; when param_2 is nonzero it calls 012ea4f0(th… |
| `0123f7c0` | UI::TournamentRequestSessionInfo::GetActionName | confirmed | Returns the address of the literal RequestSessionInfo (mov eax, imm32; ret). |
| `0123f7e0` | UI::TournamentRequestSessionInfo::GetTraceLabel | confirmed | Returns the address of the literal RequestSessionInfo : 0 : trace string (tournamenthandlers.cc:1040) (mov eax, imm32; ret). |
| `0123f7f0` | UI::TournamentRequestSessionInfo::ScalarDeletingDtor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the object, then calls _free on it when bit 0 of the flag argument is set; returns this. |
| `01241e90` | UI::TournamentRequestSessionInfo::Vslot2 | likely | Virtual slot [2]: looks up the value named SessionID through 012eb290, calls 012e0ba0, looks the object up through 01241840 and calls its vtable +0x2c with (va… |

**UI::TournamentRequestSessionLeaderboardUpdate** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123ff30` | UI::TournamentRequestSessionLeaderboardUpdate::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable then UI::TournamentRequestSessionLeaderboardUpdate::vftable into the object; when param_2 is nonzero it call… |
| `0123ffa0` | UI::TournamentRequestSessionLeaderboardUpdate::GetActionName | confirmed | Returns the address of the literal RequestSessionLeaderboardUpdate (mov eax, imm32; ret). |
| `0123ffc0` | UI::TournamentRequestSessionLeaderboardUpdate::GetTraceLabel | confirmed | Returns the address of the literal RequestSessionLeaderboardUpdate : 0 : trace string (tournamenthandlers.cc:1124) (mov eax, imm32; ret). |
| `0123ffd0` | UI::TournamentRequestSessionLeaderboardUpdate::ScalarDeleti… | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the object, then calls _free on it when bit 0 of the flag argument is set; returns this. |
| `01241f90` | UI::TournamentRequestSessionLeaderboardUpdate::Vslot2 | likely | Virtual slot [2] with one stack argument: calls 012e0ba0, looks the object up through 01241840 and calls its vtable +0xa0 with that argument when non-null; ret… |

**UI::TournamentRequestSessionsForDay** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123f600` | UI::TournamentRequestSessionsForDay::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable then UI::TournamentRequestSessionsForDay::vftable into the object; when param_2 is nonzero it calls 012ea4f0… |
| `0123f670` | UI::TournamentRequestSessionsForDay::GetActionName | confirmed | Returns the address of the literal RequestSessionsForDay (mov eax, imm32; ret). |
| `0123f690` | UI::TournamentRequestSessionsForDay::GetTraceLabel | confirmed | Returns the address of the literal RequestSessionsForDay : 0 : trace string (tournamenthandlers.cc:1020) (mov eax, imm32; ret). |
| `0123f6a0` | UI::TournamentRequestSessionsForDay::ScalarDeletingDtor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the object, then calls _free on it when bit 0 of the flag argument is set; returns this. |
| `01241e30` | UI::TournamentRequestSessionsForDay::Vslot2 | likely | Virtual slot [2]: looks up three values through 012eb290 (the key at DAT_018c9e74, the string Month, and the key at PTR_LAB_017f1d20), calls 012e0ba0, looks th… |

**UI::TournamentRequestTimedDeckSelectionDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123f210` | UI::TournamentRequestTimedDeckSelectionDisplayAction::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable then UI::TournamentRequestTimedDeckSelectionDisplayAction::vftable into the object; when param_2 is nonzero calls… |
| `0123f2a0` | UI::TournamentRequestTimedDeckSelectionDisplayAction::GetTr… | confirmed | Returns the address of the literal kRequestTimedDeckSelectionDisplayAction trace string (mov eax, imm32; ret, 6 bytes). |
| `0123f2b0` | UI::TournamentRequestTimedDeckSelectionDisplayAction::Scala… | confirmed | Stores Engine::IDisplayActionHandler::vftable into the object, then calls _free on it when bit 0 of the flag argument is set; returns this. |

**UI::TournamentRespondCurrentSessionInfoDisplayAction** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123ea30` | UI::TournamentRespondCurrentSessionInfoDisplayAction::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentRespondCurrentSessionInfoDisplayAction::vftable into the object, and when the flag argumen… |
| `0123ead0` | UI::TournamentRespondCurrentSessionInfoDisplayAction::Scala… | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
| `0123eac0` | UI::TournamentRespondCurrentSessionInfoDisplayAction::GetTr… | likely | Virtual slot [4] of UI::TournamentRespondCurrentSessionInfoDisplayAction::vftable: returns the address of its diagnostic literal. |
| `01241980` | UI::TournamentRespondCurrentSessionInfoDisplayAction::Vslot2 | likely | Virtual slot [2]: reads argument 0..3 through 012f8f60 as integers (type tag 2; asserts through 012f5a60 on mismatch, default 0) and argument 4 as a boolean (t… |

**UI::TournamentRespondEventDuelistStatsDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123f360` | UI::TournamentRespondEventDuelistStatsDisplayAction::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable then UI::TournamentRespondEventDuelistStatsDisplayAction::vftable into the object; when param_2 is nonzero calls … |
| `0123f3f0` | UI::TournamentRespondEventDuelistStatsDisplayAction::GetTra… | confirmed | Returns the address of the literal kRespondEventDuelistStatsDisplayAction trace string (mov eax, imm32; ret, 6 bytes). |
| `0123f400` | UI::TournamentRespondEventDuelistStatsDisplayAction::Scalar… | confirmed | Stores Engine::IDisplayActionHandler::vftable into the object, then calls _free on it when bit 0 of the flag argument is set; returns this. |

**UI::TournamentRespondEventLeaderBoardDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123eb80` | UI::TournamentRespondEventLeaderBoardDisplayAction::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentRespondEventLeaderBoardDisplayAction::vftable into the object, and when the flag argument … |
| `0123ec20` | UI::TournamentRespondEventLeaderBoardDisplayAction::ScalarD… | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
| `0123ec10` | UI::TournamentRespondEventLeaderBoardDisplayAction::GetTrac… | likely | Virtual slot [4] of UI::TournamentRespondEventLeaderBoardDisplayAction::vftable: returns the address of its diagnostic literal. |

**UI::TournamentRespondEventSessionInfoDisplayAction** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123e790` | UI::TournamentRespondEventSessionInfoDisplayAction::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentRespondEventSessionInfoDisplayAction::vftable into the object, and when the flag argument … |
| `0123e830` | UI::TournamentRespondEventSessionInfoDisplayAction::ScalarD… | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
| `0123e820` | UI::TournamentRespondEventSessionInfoDisplayAction::GetTrac… | likely | Virtual slot [4] of UI::TournamentRespondEventSessionInfoDisplayAction::vftable: returns the address of its diagnostic literal. |
| `01247b90` | UI::TournamentRespondEventSessionInfoDisplayAction::Forward… | likely | Virtual slot [2] of UI::TournamentRespondEventSessionInfoDisplayAction (vftable 018c9454). Reads about a dozen ValueData arguments through 012f8f60 with type c… |

**UI::TournamentRespondEventSessionsOnDayDisplayAction** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123e8e0` | UI::TournamentRespondEventSessionsOnDayDisplayAction::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentRespondEventSessionsOnDayDisplayAction::vftable into the object, and when the flag argumen… |
| `0123e980` | UI::TournamentRespondEventSessionsOnDayDisplayAction::Scala… | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
| `01246b10` | UI::TournamentRespondEventSessionsOnDayDisplayAction::Forwa… | confirmed | Virtual slot [2] of UI::TournamentRespondEventSessionsOnDayDisplayAction (vftable 018c94cc). Reads argument 0, 1 and 2 as integers through 012f8f60 (type tag 2… |
| `0123e970` | UI::TournamentRespondEventSessionsOnDayDisplayAction::GetTr… | likely | Virtual slot [4] of UI::TournamentRespondEventSessionsOnDayDisplayAction::vftable: returns the address of its diagnostic literal. |

**UI::TournamentRespondEventTotalPlayersDisplayAction** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123ef70` | UI::TournamentRespondEventTotalPlayersDisplayAction::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentRespondEventTotalPlayersDisplayAction::vftable into the object, and when the flag argument… |
| `0123f010` | UI::TournamentRespondEventTotalPlayersDisplayAction::Scalar… | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
| `0123f000` | UI::TournamentRespondEventTotalPlayersDisplayAction::GetTra… | likely | Virtual slot [4] of UI::TournamentRespondEventTotalPlayersDisplayAction::vftable: returns the address of its diagnostic literal. |
| `01241b90` | UI::TournamentRespondEventTotalPlayersDisplayAction::Vslot2 | likely | Virtual slot [2]: reads argument 0 as an integer (type tag 2) through 012f8f60 with an assert through 012f5a60 on mismatch; calls 012e0ba0, looks the object up… |

**UI::TournamentScreen** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013feb60` | UI::TournamentScreen::Slot3 | likely | Virtual slot 3 of UI::TournamentScreen::vftable and UI::TournamentScreenBase::vftable (shared body). Binds the tournament callback names JoinTournamentLobby, T… |

**UI::TournamentScreenBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013fea30` | UI::TournamentScreenBase::Ctor | confirmed | Fastcall constructor: calls base constructor 012f8420, then stores UI::TournamentScreenBase::vftable at this. |
| `013fea90` | UI::TournamentScreenBase::Dtor | confirmed | Fastcall destructor body: stores UI::TournamentScreenBase::vftable, then calls base destructor body 012f8480. |
| `013feb00` | UI::TournamentScreenBase::ScalarDeletingDtor | confirmed | Virtual slot 0 of UI::TournamentScreenBase::vftable: stores the vtable, calls base destructor body 012f8480, frees the object when the flag argument has bit 0 … |

**UI::TournamentSelectTournamentDeck** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123d4f0` | UI::TournamentSelectTournamentDeck::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable and then UI::TournamentSelectTournamentDeck::vftable into the object, and when the flag argument is nonzero … |
| `0123d590` | UI::TournamentSelectTournamentDeck::ScalarDeletingDtor | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
| `0123d560` | UI::TournamentSelectTournamentDeck::GetActionName | likely | Virtual slot [1] of UI::TournamentSelectTournamentDeck::vftable: returns the address of the literal SelectTournamentDeck. |
| `0123d580` | UI::TournamentSelectTournamentDeck::GetTraceLabel | likely | Virtual slot [4] of UI::TournamentSelectTournamentDeck::vftable: returns the address of its diagnostic literal. |
| `01243900` | UI::TournamentSelectTournamentDeck::Vslot2 | likely | Virtual slot [2] with no arguments: allocates 0x40 bytes (00d833a6), builds the string QuerySelectDeck and calls 012ec110 with it (012ec110 creates a Scaleform… |

**UI::TournamentSelectTournamentJoin** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123d3a0` | UI::TournamentSelectTournamentJoin::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable and then UI::TournamentSelectTournamentJoin::vftable into the object, and when the flag argument is nonzero … |
| `0123d440` | UI::TournamentSelectTournamentJoin::ScalarDeletingDtor | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
| `0123d410` | UI::TournamentSelectTournamentJoin::GetActionName | likely | Virtual slot [1] of UI::TournamentSelectTournamentJoin::vftable: returns the address of the literal SelectTournamentJoin. |
| `0123d430` | UI::TournamentSelectTournamentJoin::GetTraceLabel | likely | Virtual slot [4] of UI::TournamentSelectTournamentJoin::vftable: returns the address of its diagnostic literal. |
| `01243830` | UI::TournamentSelectTournamentJoin::Vslot2 | likely | Virtual slot [2] with no arguments: calls 012ebc00, names ToggleChatDialog through 01230b70, dispatches it with 012eb980 and returns 1. |

**UI::TournamentSetReadyDisplayAction** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123e4f0` | UI::TournamentSetReadyDisplayAction::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentSetReadyDisplayAction::vftable into the object, and when the flag argument is nonzero call… |
| `0123e590` | UI::TournamentSetReadyDisplayAction::ScalarDeletingDtor | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
| `0123e580` | UI::TournamentSetReadyDisplayAction::GetTraceLabel | likely | Virtual slot [4] of UI::TournamentSetReadyDisplayAction::vftable: returns the address of its diagnostic literal. |
| `01241920` | UI::TournamentSetReadyDisplayAction::Vslot2 | likely | Virtual slot [2]: calls 012e0ba0, then looks up the object through 01241840 and, when it is non-null, calls its vtable slot +0x60; returns 1. Takes no argument… |

**UI::TournamentSetupTournamentTimer** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123cb20` | UI::TournamentSetupTournamentTimer::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentSetupTournamentTimer::vftable into the object, and when the flag argument is nonzero calls… |
| `0123cbc0` | UI::TournamentSetupTournamentTimer::ScalarDeletingDtor | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set, after storing the base vtable. |
| `0123cbb0` | UI::TournamentSetupTournamentTimer::GetTraceLabel | likely | Virtual slot [4] of UI::TournamentSetupTournamentTimer::vftable: returns the address of its diagnostic literal. |
| `01242b90` | UI::TournamentSetupTournamentTimer::Vslot2 | likely | Virtual slot [2]: reads arguments 0..3 as integers (type tag 2; asserts through 012f5a60 on mismatch) and builds named values ID, Group, Round and Duration (01… |

**UI::TournamentShowStartTournamentMessage** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123c880` | UI::TournamentShowStartTournamentMessage::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentShowStartTournamentMessage::vftable into the object, and when the flag argument is nonzero… |
| `0123c920` | UI::TournamentShowStartTournamentMessage::ScalarDeletingDtor | confirmed | Stores Engine::IDisplayActionHandler::vftable into the object, then frees it through _free when bit 0 of the flag argument is set. |
| `0123c910` | UI::TournamentShowStartTournamentMessage::GetTraceLabel | likely | Virtual slot [4] of UI::TournamentShowStartTournamentMessage::vftable: returns the address of the literal kShowStartTournamentMessageDisplayAction label with i… |
| `012427d0` | UI::TournamentShowStartTournamentMessage::Vslot2 | likely | Virtual slot [2] with no arguments: builds a named bundle from the literal ShowStartTournamentMessage (0130df10), then when 00f9c010 is nonzero dispatches it t… |

**UI::TournamentShowTournamentDetails** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123d8e0` | UI::TournamentShowTournamentDetails::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable and then UI::TournamentShowTournamentDetails::vftable into the object, and when the flag argument is nonzero… |
| `0123d980` | UI::TournamentShowTournamentDetails::ScalarDeletingDtor | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
| `0123d950` | UI::TournamentShowTournamentDetails::GetActionName | likely | Virtual slot [1] of UI::TournamentShowTournamentDetails::vftable: returns the address of the literal ShowTournamentDetails. |
| `0123d970` | UI::TournamentShowTournamentDetails::GetTraceLabel | likely | Virtual slot [4] of UI::TournamentShowTournamentDetails::vftable: returns the address of its diagnostic literal. |
| `01241110` | UI::TournamentShowTournamentDetails::Vslot2 | likely | Virtual slot [2]: looks up the object named Lobby via 012eb290, walks its element vector (4-byte items at +0x58..+0x5c) calling the element predicate at vtable… |

**UI::TournamentShowTournamentScores** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123cdc0` | UI::TournamentShowTournamentScores::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentShowTournamentScores::vftable into the object, and when the flag argument is nonzero calls… |
| `0123ce60` | UI::TournamentShowTournamentScores::ScalarDeletingDtor | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set, after storing the base vtable. |
| `01245400` | UI::TournamentShowTournamentScores::BuildScoresRecord | confirmed | Virtual slot [2] of UI::TournamentShowTournamentScores (vftable 018c8b18). Reads eight fields from the argument list through 012f8f60: integers at indices 0, 1… |
| `0123ce50` | UI::TournamentShowTournamentScores::GetTraceLabel | likely | Virtual slot [4] of UI::TournamentShowTournamentScores::vftable: returns the address of its diagnostic literal. |

**UI::TournamentStartTournament** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123c730` | UI::TournamentStartTournament::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentStartTournament::vftable into the object, and when the flag argument is nonzero calls FUN_… |
| `0123c7d0` | UI::TournamentStartTournament::ScalarDeletingDtor | confirmed | Stores Engine::IDisplayActionHandler::vftable into the object, then frees it through _free when bit 0 of the flag argument is set. |
| `0123c7c0` | UI::TournamentStartTournament::GetTraceLabel | likely | Virtual slot [4] of UI::TournamentStartTournament::vftable: returns the address of the literal kStartTournamentRoundDisplayAction label with its source file an… |
| `012424c0` | UI::TournamentStartTournament::Vslot2 | likely | Virtual slot [2]: reads arguments 0 and 1 as integers (type tag 2) through 012f8f60, asserting through 012f5a60 on mismatch; builds named values ID and Round (… |

**UI::TournamentTournamentLobbyList** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123d250` | UI::TournamentTournamentLobbyList::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable and then UI::TournamentTournamentLobbyList::vftable into the object, and when the flag argument is nonzero c… |
| `0123d2f0` | UI::TournamentTournamentLobbyList::ScalarDeletingDtor | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
| `0123d2c0` | UI::TournamentTournamentLobbyList::GetActionName | likely | Virtual slot [1] of UI::TournamentTournamentLobbyList::vftable: returns the address of the literal TournamentLobbyList. |
| `0123d2e0` | UI::TournamentTournamentLobbyList::GetTraceLabel | likely | Virtual slot [4] of UI::TournamentTournamentLobbyList::vftable: returns the address of its diagnostic literal. |

**UI::TournamentUpdateTournament** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123c5e0` | UI::TournamentUpdateTournament::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentUpdateTournament::vftable into the object, and when the flag argument is nonzero calls FUN… |
| `0123c680` | UI::TournamentUpdateTournament::ScalarDeletingDtor | confirmed | Stores Engine::IDisplayActionHandler::vftable into the object (the derived vtable store is absent in this body), then frees the object through _free when bit 0… |
| `01244b00` | UI::TournamentUpdateTournament::BuildStateValue | confirmed | Virtual slot [2] of UI::TournamentUpdateTournament (vftable 018c8854). Reads seven fields from the argument list through 012f8f60: integers at indices 0, 1, 2 … |
| `0123c670` | UI::TournamentUpdateTournament::GetTraceLabel | likely | Virtual slot [4] of UI::TournamentUpdateTournament::vftable: returns the address of the literal kUpdateTournamentDisplayAction label with its source file and l… |

**UI::TournamentUpdateTournamentTimer** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123cc70` | UI::TournamentUpdateTournamentTimer::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentUpdateTournamentTimer::vftable into the object, and when the flag argument is nonzero call… |
| `0123cd10` | UI::TournamentUpdateTournamentTimer::ScalarDeletingDtor | confirmed | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set, after storing the base vtable. |
| `0123cd00` | UI::TournamentUpdateTournamentTimer::GetTraceLabel | likely | Virtual slot [4] of UI::TournamentUpdateTournamentTimer::vftable: returns the address of its diagnostic literal. |
| `01242ff0` | UI::TournamentUpdateTournamentTimer::Vslot2 | likely | Virtual slot [2]: reads arguments 0..3 as integers (type tag 2) and builds named values ID, Group, Round and Duration inside the bundle named UpdateTournamentT… |

**(no class)** (17)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0125d190` | QueueSwapBaseCardWithHandCard | confirmed | Builds two Utility::XmlBuilder element lists with <Elements> and card element fields (0130df10, 0131e290, 0130e120, 0130e200, 0130e6c0), creates an Invoke Scal… |
| `0125d510` | QueueDiscardHandCard | confirmed | Creates an Invoke ScaleformAction whose MethodName is DiscardHandCard (012ec220), adds the integer argument parameter_1 under the key CardElementID (012ebe40),… |
| `0125d5d0` | QueueScanCard | confirmed | Creates an Invoke ScaleformAction whose MethodName is ScanCard (012ec220), then copies the int vector at object+4/+8 into an <Elements><Element>id</Element>...… |
| `012610e0` | QueueOpenSelectBaseCardDialog | confirmed | Builds XML text with <Flags> holding myPlayer and myBase (1 or 0, from comparing param_2 and param_1 with the thread-owned object's field +0x10) and <Elements>… |
| `01261660` | QueueOpenSelectCardDialog | confirmed | Builds an <Elements> XML text with one card element per int entry of the vector at param_2 (0040aa9a count, 01259110 tree lookup, 0131e290 per card); queues Op… |
| `0131eae0` | ActionButtonXmlWrite | confirmed | Void, seven stack args. Writes a <Button> element: an ActionType int child (arg2) through 0130e7b0; when arg6 (flag byte) is nonzero it also writes ElementID (… |
| `013e52d0` | LeaderboardEntry_SerializeXml | confirmed | Thiscall with the leaderboard row at param_1: +0 Position (int), +4 CharacterName (string), +0x20 ObjectiveScore, +0x24 TotalGames, +0x28 TotalWins, +0x2c Tota… |
| `013e5f70` | EventDay_SerializeXml | confirmed | Thiscall with an event-day record at param_1: int +0 (Year), int +4 (Month), and an int at +8 written under the 3-character key Day (bytes 44 61 79 at .rdata 0… |
| `01230390` | UI::TradingCard::InitMainMovie | likely | Stores param_2 at this+0xc, calls a virtual on param_2 (vtable +0xe8, slot +0x14) with ID 0x1c and the string .\UI\TradingCard\TCCW_Main.swf, calls the same vi… |
| `0125d890` | QueueEffectPlayAnimation | likely | Reads two ValueData arguments through 01357a00 and a vtable call with property 0x1ae03; in the false branch it type-checks an integer list (kIntegerListTypeID … |
| `012622e0` | BuildCardElementParams | likely | Builds a parameter table for one card element, inserting in order the keys Button, Image, Title, WhiteCost, RedCost, BlueCost, GreenCost, BlackCost, ColorlessC… |
| `0131ccd0` | SetField50 | likely | Stores the stack argument dword into this+0x50 (mov eax,[esp+4]; mov [ecx+0x50],eax) and returns 4 bytes of arguments. |
| `0131f0e0` | CardAttributeXmlWrite | likely | Thiscall (ECX = object with a service pointer at +0xc), args: p2 unused by the body, p3 = attribute id, p4 = ValueData pointer. Names the child with 0131cee0(p… |
| `0131f550` | DispatchModifyCardFromVector58 | likely | Fastcall (ECX = object), works on the int vector at this+0x58..0x5c. When it is non-empty and its first element is 1 it reads the second and third elements and… |
| `01392560` | SetCardReferenceText | likely | Fastcall with ECX = card. Recomputes the reference text at this+0x124. If this+0x64 is zero and vtable slot 122 (+0x1e8) is false it assigns 'a card' (6 bytes)… |
| `01392800` | Card::FormatPossessiveText | likely | Thiscall with one output string argument. If vtable slot 122 is true and FUN_01350660(this+0x40) is non-zero it calls 01346170 on the output. Otherwise it copi… |
| `013be8e0` | TcgContextMenu_Build | likely | Builds the player context-menu XML tree for target account param_2: opens an XmlBuilder (FUN_0130e6c0), then runs an 8-step loop. Step 0 (FUN_012ce190 false) a… |

## Source shards

[[S0269 009a6cf0-009aceb0|S0269]] (6), [[S0322 00acd6d0-00ad3250|S0322]] (30), [[S0542|S0542]] (1), [[S0595|S0595]] (62), [[S0652|S0652]] (1), [[S0654 012393a0-0123f010|S0654]] (112), [[S0655|S0655]] (80), [[S0656|S0656]] (12), [[S0658 0124dfe0-01253ee0|S0658]] (2), [[S0660|S0660]] (44), [[S0661 0125d190-012619f0|S0661]] (6), [[S0662|S0662]] (7), [[S0664|S0664]] (17), [[S0665 01271690-01277340|S0665]] (93), [[S0678|S0678]] (19), [[S0679|S0679]] (14), [[S0684|S0684]] (24), [[S0685|S0685]] (1), [[S0688|S0688]] (3), [[S0689|S0689]] (3), [[S0691|S0691]] (19), [[S0692|S0692]] (4), [[S0693|S0693]] (12), [[S0695|S0695]] (12), [[S0696|S0696]] (20), [[S0697|S0697]] (7), [[S0698|S0698]] (1), [[S0699|S0699]] (18), [[S0700|S0700]] (16), [[S0701|S0701]] (22), [[S0702|S0702]] (20), [[S0703|S0703]] (21), [[S0704|S0704]] (63), [[S0706|S0706]] (3), [[S0707|S0707]] (6), [[S0708|S0708]] (14), [[S0709|S0709]] (39), [[S0711 01369c70-0136dbd0|S0711]] (23), [[S0712|S0712]] (30), [[S0713|S0713]] (40), [[S0714|S0714]] (23), [[S0715|S0715]] (35), [[S0716|S0716]] (111), [[S0717|S0717]] (23), [[S0718|S0718]] (23), [[S0719|S0719]] (36), [[S0720|S0720]] (62), [[S0721|S0721]] (3), [[S0723|S0723]] (20), [[S0727 013be8e0-013c3b90|S0727]] (1), [[S0730|S0730]] (80), [[S0731|S0731]] (8), [[S0734|S0734]] (19), [[S0735|S0735]] (27), [[S0738|S0738]] (19), [[S0740|S0740]] (4), [[S0741|S0741]] (34), [[S0742|S0742]] (58), [[S0743|S0743]] (48), [[S0745|S0745]] (38), [[S0746|S0746]] (45), [[S0747|S0747]] (76), [[S0749|S0749]] (10), [[S0751|S0751]] (19), [[S0752|S0752]] (6), [[S0754|S0754]] (45), [[S0755|S0755]] (72), [[S0757|S0757]] (25), [[S0758|S0758]] (76), [[S0760|S0760]] (81), [[S0761|S0761]] (55), [[S0762|S0762]] (28), [[S0766|S0766]] (60), [[S0767|S0767]] (134), [[S0768|S0768]] (169), [[S0769|S0769]] (42), [[S0774|S0774]] (27), [[S0781|S0781]] (17), [[S0785|S0785]] (42), [[S0786|S0786]] (184), [[S0787|S0787]] (182), [[S0788|S0788]] (180), [[S0789|S0789]] (177), [[S0790|S0790]] (44), [[S0791|S0791]] (14), [[S0797|S0797]] (3)
