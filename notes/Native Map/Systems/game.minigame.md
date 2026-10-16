# System: game.minigame

67 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (18)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009b6950` | `OnMiniGameObjectiveFailed` |  | Applies status 4 locally and conditionally reports ObjectiveFailed to the UI. | [[Native Map/Shards/S0271 009b2210-009b7170|S0271]] |
| `009b9500` | `MiniGameState_StartActive` |  | Begin the active mini-game/encounter state without sending a packet; the vault note records the client running this on the 41/2 encounter-enter success path (00aa38c6, with the manager pointer taken from player+0xf8) as… | [[Native Map/Shards/S0272 009b7660-009bd590|S0272]] |
| `009bc410` | `ClientMiniGameManager::ClientMiniGameManager` | ClientMiniGameManager | Build the client-side mini-game manager: its UI handlers, definition tables and the active-state list used by every handler in this shard. | [[Native Map/Shards/S0272 009b7660-009bd590|S0272]] |
| `009bd650` | `StartFlashMiniGameUi` |  | Presents the flash mini-game in its normal or micro view and requests dismount when transport is available. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009be7d0` | `RequestStartMiniGame` |  | Requests that the server start the pending mini-game after local setup completes. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009beb70` | `HandleMiniGameStart` |  | Applies a received mini-game start record and prepares local UI and state. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009bef50` | `CancelMiniGame` |  | Cancels the current mini-game invitation or active state and restores client UI state. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009bf0f0` | `LeaveMiniGameInstance` |  | Leaves the requested mini-game instance or the current state when eligible; the fixed log does not include the requested id. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009bf190` | `LeaveMiniGame` |  | Cleans up a matching mini-game state and emits local end-screen/UI operations; its server-directed role comes from caller 009bf890 case 0x13. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009bf5a0` | `UpdateMiniGameManagerState` | ClientMiniGameManager | Updates mini-game manager state and conditionally starts the pending game. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009bf5e0` | `TickMiniGameManager` |  | Advances client mini-game lifecycle and loading state each update. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `00ad32e0` | `SetPair` |  | Shared virtual implementation in ClientSimpleCardGameManager and ClientTradingCardGameManager. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00ad3670` | `ClientTradingCardGameManager::InitializeAssets` | ClientTradingCardGameManager | Initializes TCG assets and records related state for ClientTradingCardGameManager. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00ad3740` | `ClientTradingCardGameManager::OnDisconnect` | ClientTradingCardGameManager | Trading card manager disconnect cleanup/logging. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00ad3e40` | `ClientTradingCardGameManager::Shutdown` | ClientTradingCardGameManager | Shuts down trading card game manager resources and state. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00ad4f50` | `ClientTradingCardGameManager::Update` | ClientTradingCardGameManager | Ticks the trading card game manager's startup/load phases and child jobs. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00ad6410` | `ClientTradingCardGameManager::Dtor` | ClientTradingCardGameManager | Destroys manager-owned TCG state and resources. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00ad6510` | `ClientTradingCardGameManager::ScalarDeletingDtor` | ClientTradingCardGameManager | Scalar deleting destructor for ClientTradingCardGameManager. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |

## 🟢 likely (32)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009b2f60` | `MiniGameDataRecordCtor` |  | Initializes a mini-game data record used by the manager's data-table loader. | [[Native Map/Shards/S0271 009b2210-009b7170|S0271]] |
| `009b3050` | `MiniGameScoreInfoCtor` |  | Constructs an element of the array used by the mini-game end-score packet. | [[Native Map/Shards/S0271 009b2210-009b7170|S0271]] |
| `009b4d00` | `ParseMiniGameGroupTable` |  | Loads the mini-game group data table. | [[Native Map/Shards/S0271 009b2210-009b7170|S0271]] |
| `009b5fd0` | `InitGroupElementArrayAndStrings → MiniGameGroupInfoRecordInit` |  | Initializes embedded group-info fields used by the packet constructor; record role is inferred from callers. | [[Native Map/Shards/S0271 009b2210-009b7170|S0271]] |
| `009b6070` | `DestroyGroupElementArrayAndStrings → MiniGameGroupInfoRecordDtor` |  | Destroys embedded group-info fields; packet-record role is inferred from caller 009b6140. | [[Native Map/Shards/S0271 009b2210-009b7170|S0271]] |
| `009b62a0` | `LoadMiniGameDataTables ⚠downgrade` |  | Loads the mini-game catalogue tables during manager initialization. | [[Native Map/Shards/S0271 009b2210-009b7170|S0271]] |
| `009b6700` | `OnMiniGameObjectiveComplete ⚠downgrade` |  | Applies a minigame objective completion to local state and UI. | [[Native Map/Shards/S0271 009b2210-009b7170|S0271]] |
| `009b8010` | `MiniGameState_Destroy` |  | Destroy one mini-game state record - the nodes that 009baba0 unlinks from the manager's HashListMap<int,MiniGameState> all end in this destructor, which is why the role is inferred from that caller and from the MiniGame… | [[Native Map/Shards/S0272 009b7660-009bd590|S0272]] |
| `009b8190` | `MiniGameState_InitWithKey` |  | Give the new state node created by 009b9680 its key before it is inserted into the manager's state hash list. | [[Native Map/Shards/S0272 009b7660-009bd590|S0272]] |
| `009b8bc0` | `MiniGameGroupRecord_Copy` |  | Copy the 76-byte record decoded by 009b9860 into the manager's MiniGameGroup storage before the group-info applier 009b9fc0 publishes its fields. | [[Native Map/Shards/S0272 009b7660-009bd590|S0272]] |
| `009b9680` | `MiniGameState_CreateAndInsert` |  | Create and link the next active-state node while a status message replaces the current state (called by HandlerMiniGameStatus_Populate 009bb2d0 with the previous state's id). | [[Native Map/Shards/S0272 009b7660-009bd590|S0272]] |
| `009b9780` | `MiniGameState_DestroyThunk` |  | Destructor entry point for the state class destroyed by 009b8010; it has no recorded direct caller, so it is reached through a function pointer or table. | [[Native Map/Shards/S0272 009b7660-009bd590|S0272]] |
| `009b9b30` | `ObjectiveDataState_Construct` |  | Initialise the state/objective record that the MiniGameInfoPacket constructor 009b9c70 and the MiniGameUpdateRewardPacket constructor 009b9eb0 embed at +0x18 of their packets. | [[Native Map/Shards/S0272 009b7660-009bd590|S0272]] |
| `009baba0` | `MiniGameStateList_RemoveNode` |  | Remove and destroy one active mini-game state node; called by the leave/close handler 009bd240, the status-populate handler 009bb2d0 and the list drain routines. | [[Native Map/Shards/S0272 009b7660-009bd590|S0272]] |
| `009bace0` | `MiniGameStateList_Clear` |  | Empty a list whose head pointer lives at this+8 by repeatedly removing the head node through 009baba0; no caller is recorded. | [[Native Map/Shards/S0272 009b7660-009bd590|S0272]] |
| `009bca60` | `ClientMiniGameManager::ScalarDeletingDestructor` | ClientMiniGameManager | Virtual deleting destructor for ClientMiniGameManager: this thunk is slot 0 of the vtable at 0180f130 (ClientMiniGameManager::vftable), read directly with nmq bytes - that is where the class binding comes from. | [[Native Map/Shards/S0272 009b7660-009bd590|S0272]] |
| `009bda80` | `SendLootWheelRotationStopped` |  | Reports completion of the loot-wheel rotation interaction. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009bdaf0` | `RequestTcgChallenge` |  | Requests a trading-card-game challenge after local guard checks pass. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009bdbd0` | `RequestWheelInteraction` |  | Starts the wheel interaction only when the local activity is available. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009bdc60` | `SendMiniGamePacket` |  | Shared client sender for mini-game control packets. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009be8e0` | `SendMiniGameLeaveSubop6` |  | Requests the mini-game leave operation through the shared sender. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009be8f0` | `SendMiniGameLeaveSubop7` |  | Requests the mini-game state-leave operation for the given id. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009be910` | `SetMiniGameReady` |  | Marks the client ready for a mini-game or takes the special local game-type path. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009be970` | `SetMiniGameNotReady` |  | Marks the client not ready for a mini-game. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009be9d0` | `ToggleMiniGameReady` |  | Toggles the local ready flag and notifies the mini-game server. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009bea10` | `SendMiniGameSubop25` |  | Sends the mini-game operation represented by sub-opcode 0x19. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009bea40` | `SendMiniGameSubop26` |  | Sends the mini-game operation represented by sub-opcode 0x1a. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009bf4d0` | `ApplyMiniGameLeaveState` |  | Applies a mini-game leave result to current local state. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `009bf590` | `SendMiniGameSubop37` |  | Sends the mini-game operation represented by sub-opcode 0x25. | [[Native Map/Shards/S0273 009bd650-009c2c50|S0273]] |
| `00ad1270` | `SetFlag5ef62` |  | Sets the manager's ready-related flag; called by the minigame ready handler. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad12a0` | `ClearFlag5ef62` |  | Clears the ready-related flag in the minigame handler path. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |
| `00ad12d0` | `SetFlag5ef61IfHandlerPresent → ClientSimpleCardGameManager::SetFlag5ef61IfHandl…` |  | Sets a second handler state flag in the local minigame path. | [[Native Map/Shards/S0322 00acd6d0-00ad3250|S0322]] |

## 🟡 speculative (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009ba4c0` | `RewardEntry110_Copy ? ⚠rename` |  | Element copy used by the 0x110-stride array reallocation 009bad20 and the reward append 009bc140: a reward row with three wide strings and an embedded reward bundle. | [[Native Map/Shards/S0272 009b7660-009bd590|S0272]] |
| `00ad33f0` | `GetEnabledState ? ⚠rename` |  | Shared virtual implementation in ClientSimpleCardGameManager and ClientTradingCardGameManager. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00ad3440` | `GetMusicLevels ? ⚠rename` |  | Shared virtual implementation in ClientSimpleCardGameManager and ClientTradingCardGameManager; reads two music setting values. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00ad4340` | `GetCardCount ? ⚠rename` |  | Shared card manager query whose exact input meaning remains unresolved. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00ad4390` | `SelectCard ? ⚠rename` |  | Shared card manager operation over a card-related object; precise selection semantics unresolved. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |

## ⚪ unknown (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00939100` | `— ⚠reject` |  | Selects ordinary minigame quit only when the mode query succeeds but its state is missing or has byte +0x61 clear; other paths issue the encounter-exit request. The address-citing lifecycle note associates this UI actio… | [[Native Map/Shards/S0247 00934740-00939200|S0247]] |
| `009b3290` | `— ⚠reject` |  | Parses MiniGameData.txt as routed by caller 009b62a0; table identity rests on that loader call. | [[Native Map/Shards/S0271 009b2210-009b7170|S0271]] |
| `009b34f0` | `— ⚠reject` |  | Parses MiniGameTypeData.txt as routed by caller 009b62a0; table identity rests on that loader call. | [[Native Map/Shards/S0271 009b2210-009b7170|S0271]] |
| `009b3710` | `— ⚠reject` |  | Parses MiniGameGroupLinks.txt as routed by caller 009b62a0; table identity rests on that loader call. | [[Native Map/Shards/S0271 009b2210-009b7170|S0271]] |
| `009b4ca0` | `— ⚠reject` |  | Called while 00c42c60 rebuilds a record array; record identity comes from that caller, while this helper establishes only the zeroed offsets and released +0x70 member. | [[Native Map/Shards/S0271 009b2210-009b7170|S0271]] |
| `009b4f60` | `—` |  | Walks nodes from manager+0x2c0d4 via next at node+0xc. For each node, hashes its dword at +0x4c modulo 100 and searches the first bucket table at manager+0x2c5c0. If a matching key is found, scans the second table at ma… | [[Native Map/Shards/S0271 009b2210-009b7170|S0271]] |
| `009ba330` | `— ⚠reject` |  | Virtual deleting destructor for the state class destroyed by 009b8010 (the nodes removed from the manager's state list). | [[Native Map/Shards/S0272 009b7660-009bd590|S0272]] |
| `009ba390` | `— ⚠reject` |  | Element copy used by the 0xe8-stride array reallocation 009ba5c0 and by the reward-list readers, so the element embeds a RewardBundleBase and a byte flag at the end. | [[Native Map/Shards/S0272 009b7660-009bd590|S0272]] |
| `009bbba0` | `— ⚠reject` | ClientMiniGameManager | Release the whole client mini-game manager: its definition tables, UI sub-handlers, strings and state list. | [[Native Map/Shards/S0272 009b7660-009bd590|S0272]] |
| `00ad4510` | `— ⚠reject` |  | Shared card manager lookup/insert operation. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00ad47f0` | `— ⚠reject` | ClientTradingCardGameManager | Startup for the trading card game manager. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00ad6300` | `— ⚠reject` | ClientTradingCardGameManager | Constructs the trading card game manager and its asset-loading state. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `009bd650` | `StartFlashMiniGameUi` | send | PacketDismountRequest |  |  |
| `009beb70` | `HandleMiniGameStart` | send | PacketDismountRequest |  |  |
| `00939100` | `— ⚠reject` | send | BaseEncounterPacket |  |  |
