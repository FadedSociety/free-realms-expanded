# System: ui.minigame

86 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (46)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009aae90` | `MiniGameFlash_EmitOnDisconnect` |  | Disconnect notification for the minigame flash/script layer (the event name matches the Lua MiniGameFlash:OnDisconnect handler); called from 00904560. | [[Native Map/Shards/S0269 009a6cf0-009aceb0|S0269]] |
| `009aaeb0` | `MiniGameToolbarThunk_00c44160` |  | Toolbar activation entry for active minigames: the vault's combat-controls note records the type-3 dispatch 009aaeb0 -> 00c44160 as the minigame toolbar. | [[Native Map/Shards/S0269 009a6cf0-009aceb0|S0269]] |
| `009aaef0` | `MiniGameShowStageSelect_Emit` |  | Stage-select presentation for the minigame start handler (both literal event names are the anchors); called from the minigame packet builder 009bfea0. | [[Native Map/Shards/S0269 009a6cf0-009aceb0|S0269]] |
| `009b3970` | `ShowCollectedMiniGameItemRewards` |  | Displays collected mini-game item rewards. | [[Native Map/Shards/S0271 009b2210-009b7170|S0271]] |
| `009b6aa0` | `ShowMiniGameHud` |  | Shows the game-specific HUD while a minigame is active. | [[Native Map/Shards/S0271 009b2210-009b7170|S0271]] |
| `009b6da0` | `StopMiniGameLootWheelAt` |  | Stops the results-screen loot wheel on a selected reward. | [[Native Map/Shards/S0271 009b2210-009b7170|S0271]] |
| `009b6fe0` | `ShowMiniGameMarketplaceAd` |  | Requests display of the marketplace ad in the minigame UI. | [[Native Map/Shards/S0271 009b2210-009b7170|S0271]] |
| `009b7660` | `HandlerMiniGameStart_SetGroupMemberSelectingJob` |  | Fire the Lua handler HandlerMiniGameStart:setGroupMemberSelectingJob for the group member identified by the two id arguments; param_4 becomes the selecting flag passed to Lua. | [[Native Map/Shards/S0272 009b7660-009bd590|S0272]] |
| `009b7810` | `NpxPlayMore_Show` |  | Build the four Lua arguments for NpxPlayMore:Show from an activity category and three strings, then dispatch the promo-window event. | [[Native Map/Shards/S0272 009b7660-009bd590|S0272]] |
| `009b79d0` | `VerifyPlayMoreWindow` |  | Flush a pending 'play more' request into the NpxPlayMore window once the caller (param_2) or the stored flag allows showing it. | [[Native Map/Shards/S0272 009b7660-009bd590|S0272]] |
| `00a69870` | `PiratesShipSelection_ConfirmSelection` |  | Button handler of the Pirates ship-selection screen: the client tells the server the chosen ship name and closes the Flash screen. | [[Native Map/Shards/S0303 00a674c0-00a6c3b0|S0303]] |
| `00ad3b40` | `ReleaseTcgPointer` |  | Shared virtual implementation for both card manager classes; releases pointer capture on the TCG window. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00ad3fd0` | `CreateTcgWindow` |  | Shared virtual implementation in both card manager classes; creates the TCG main window. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00c07e50` | `RegisterMiniGameModule` |  | Registers the client 'MiniGame' Lua module (TogglePause/Pause/Unpause, InviteToMiniGame, RestartMiniGame, EndGameDialogClosed, SetSelectedMiniGameDefinitionId, SetHowToVisible, IsInMiniGame/IsInFlashMiniGame/IsInArena). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c08280` | `RegisterMiniGameFlashCModule` |  | Registers the native side of the Flash minigame host module 'MiniGameFlashC' (LoadData, SendMessage, ChessInit, ChessMovePlayer, ChessMoveRequest, StartWheel, RequestTintByName) that ScriptsBase.lua calls through MiniGa… | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c08570` | `MiniGameFlash_RequestTintByName` |  | Handler registered as 'RequestTintByName' in the MiniGameFlashC module (0x0182a1e8): asks the flash host to tint the named object; the embedded string names the action. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c45760` | `MiniGameGoalDataSource_GetColumnName` | MiniGameGoalDataSource | Names the columns the UI/Lua table reads through BaseClient.MiniGameGoals. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c45800` | `MiniGameScoreDataSource_GetColumnName` | MiniGameScoreDataSource | Names the columns of the per-minigame score rows exposed through BaseClient.MiniGameScore. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c45860` | `MiniGameGroupDataSource_GetColumnName` | MiniGameGroupDataSource | Names the columns of the minigame-group catalogue rows exposed through BaseClient.MiniGameGroup. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c45a00` | `MiniGameDataSource_GetColumnName` | MiniGameDataSource | Names the columns of the live minigame row exposed through BaseClient.MiniGame. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c45af0` | `MiniGameDataDataSource_GetColumnName` | MiniGameDataDataSource | Names the columns of the static minigame definition rows exposed through BaseClient.MiniGameData. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c45bb0` | `MiniGameAchievementsDataSource_GetColumnName` | MiniGameAchievementsDataSource | Names the columns of the achievements/trophies rows exposed through BaseClient.MiniGame.Trophies. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c45e20` | `MiniGameGoalDataSource_GetRowCount` | MiniGameGoalDataSource | Tells the table how many goal rows exist for BaseClient.MiniGameGoals. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c45e40` | `MiniGameScoreDataSource_GetRowCount` | MiniGameScoreDataSource | Tells the table how many score rows exist for BaseClient.MiniGameScore. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c460e0` | `MiniGameGoalDataSource_GetValue` | MiniGameGoalDataSource | Serves one cell of the BaseClient.MiniGameGoals table, including the category prefix and the human state string. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c468c0` | `MiniGameDataSource_GetValue` | MiniGameDataSource | Serves the BaseClient.MiniGame (and BaseClient.LastMiniGame) table row that the minigame browser and end-of-game dialog read. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c46c20` | `MiniGameDataDataSource_GetValue` | MiniGameDataDataSource | Serves the BaseClient.MiniGameData table used when the UI needs the static definition of a minigame. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c46e40` | `MiniGameAchievementsDataSource_GetValue` | MiniGameAchievementsDataSource | Serves the BaseClient.MiniGame.Trophies table cells; the '<not found>' default is visible whenever a trophy row cannot be resolved. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c46f30` | `MiniGameGoalDataSource_Destruct` | MiniGameGoalDataSource | Tear down the BaseClient.MiniGameGoals data source and its index list. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c46fc0` | `MiniGameScoreDataSource_Destruct` | MiniGameScoreDataSource | Tear down the BaseClient.MiniGameScore data source. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c47030` | `MiniGameGroupDataSource_Destruct` | MiniGameGroupDataSource | Tear down the BaseClient.MiniGameGroup data source. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c470c0` | `MiniGameDataSource_Destruct` | MiniGameDataSource | Tear down BaseClient.MiniGame / BaseClient.LastMiniGame and their attached goal source. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c47180` | `MiniGameDataDataSource_Destruct` | MiniGameDataDataSource | Tear down the BaseClient.MiniGameData data source. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c47210` | `MiniGameDataDataSource_GetRowCount` | MiniGameDataDataSource | Tells the table how many definition rows exist for BaseClient.MiniGameData. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c472d0` | `MiniGameGoalDataSource_Construct` | MiniGameGoalDataSource | Build the BaseClient.MiniGameGoals table source attached to the MiniGameDataSource (allocated with size 0x270 by 00c474e0). | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c473a0` | `MiniGameScoreDataSource_Construct` | MiniGameScoreDataSource | Build the BaseClient.MiniGameScore table source; called from the minigame-manager constructor 009bc410. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c47440` | `MiniGameGroupDataSource_Construct` | MiniGameGroupDataSource | Build the BaseClient.MiniGameGroup table source; called from the minigame-manager constructor 009bc410. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c474e0` | `MiniGameDataSource_Construct` | MiniGameDataSource | Build the BaseClient.MiniGame (and BaseClient.LastMiniGame) table source, including the goal sub-source the UI reads after each minigame. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c475e0` | `MiniGameDataDataSource_Construct` | MiniGameDataDataSource | Build the BaseClient.MiniGameData table source; called from the minigame-manager constructor 009bc410. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c47680` | `MiniGameAchievementsDataSource_ConstructTrophies` | MiniGameAchievementsDataSource | Build the BaseClient.MiniGame.Trophies table source; called from the minigame-manager constructor 009bc410. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c477c0` | `MiniGamePortalDataSource_GetColumnName` | MiniGamePortalDataSource | Names the columns of the minigame portal/browser rows exposed through BaseClient.MiniGamePortal. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c47930` | `MiniGamePortalDataSource_GetValue` | MiniGamePortalDataSource | Serves the portal/browser rows the minigame selection UI displays, including each portal's lock status. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c47d00` | `MiniGamePortalDataSource_Construct` | MiniGamePortalDataSource | Build the BaseClient.MiniGamePortal table source; called from the minigame-manager constructor 009bc410. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c48040` | `MiniGameBossDataSource_GetColumnName` | MiniGameBossDataSource | Names the two columns of the boss rows the minigame encounter UI shows. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c488d0` | `MiniGameBossDataSource_GetValue` | MiniGameBossDataSource | Typed column read that feeds the Flash UI showing the 'MiniGame.Bosses' rows: a name column and a numeric value column. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c48e00` | `MiniGameBossDataSource::MiniGameBossDataSource` | MiniGameBossDataSource | Creates the 'MiniGame.Bosses' UI data source that later receives boss rows from boss messages. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |

## 🟢 likely (27)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009aaec0` | `AdjustorThunk_add58_009aaec0` |  | Sibling of the toolbar thunks in this group (same +0x58 adjustor as FUN_009aaeb0); called from 00a33760 in the minigame/UI range. | [[Native Map/Shards/S0269 009a6cf0-009aceb0|S0269]] |
| `009aaed0` | `AdjustorThunk_add58_009aaed0` |  | Sibling of the toolbar thunks in this group (same +0x58 adjustor); called from 00a2d820/00a2e6b0/00a35cc0. | [[Native Map/Shards/S0269 009a6cf0-009aceb0|S0269]] |
| `009aaee0` | `AdjustorThunk_add58_009aaee0` |  | Sibling of the toolbar thunks in this group (same +0x58 adjustor); called from 00a2d790. | [[Native Map/Shards/S0269 009a6cf0-009aceb0|S0269]] |
| `009b3c30` | `PreparePlayMorePrompt` |  | Prepares localized play-more prompt data; this function does not dispatch or show the prompt. | [[Native Map/Shards/S0271 009b2210-009b7170|S0271]] |
| `00c082d0` | `MiniGameFlash_LoadData` |  | Handler registered as 'LoadData' in the MiniGameFlashC module (0x0182a1e8): loads the minigame data blob the flash movie asked for. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c45b90` | `MiniGameAchievementsDataSource_PopulateOnDemand` | MiniGameAchievementsDataSource | Lazy population/refresh hook of the trophies data source before its base accessor runs. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c45e60` | `MarkAttachedSourceDirtyAndNotify` |  | Invalidate the cached goal-row index when minigame state changes; called from the minigame state-change paths 009bb2d0, 009bd8f0, 009bd970 and 009beb70. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c45f80` | `MiniGameAchievementsDataSource_PushAllRows` | MiniGameAchievementsDataSource | Bulk row push of the trophies data source; the populated byte is cleared again by 00c472a0/00c47680 when the source changes. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c460c0` | `FormatStateValueIntoBuffer` |  | Render the goal 'State' column text used by MiniGameGoalDataSource_GetValue. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c462a0` | `MiniGameScoreDataSource_GetValue ⚠downgrade` | MiniGameScoreDataSource | Serves one cell of the BaseClient.MiniGameScore table (icon id, score type, count, max, points). | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c472a0` | `MiniGameAchievementsDataSource_SetSource` | MiniGameAchievementsDataSource | Change the achievements source and invalidate/refresh the cached rows. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c47380` | `MiniGameGoalDataSource_ScalarDeletingDtor` | MiniGameGoalDataSource | Scalar deleting destructor of the BaseClient.MiniGameGoals data source. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c47420` | `MiniGameScoreDataSource_ScalarDeletingDtor` | MiniGameScoreDataSource | Scalar deleting destructor of the BaseClient.MiniGameScore data source. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c474c0` | `MiniGameGroupDataSource_ScalarDeletingDtor` | MiniGameGroupDataSource | Scalar deleting destructor of the BaseClient.MiniGameGroup data source. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c475c0` | `MiniGameDataSource_ScalarDeletingDtor` | MiniGameDataSource | Scalar deleting destructor of the BaseClient.MiniGame data source. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c47660` | `MiniGameDataDataSource_ScalarDeletingDtor` | MiniGameDataDataSource | Scalar deleting destructor of the BaseClient.MiniGameData data source. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c47720` | `MiniGameAchievementsDataSourceA_ScalarDeletingDtor` | MiniGameAchievementsDataSource | Scalar deleting destructor paired with MiniGameAchievementsDataSource_ConstructA. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c478b0` | `PortalRowSet_PushAllToSink` |  | Populate the BaseClient.MiniGamePortal table before its row count/values are read. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c47910` | `MiniGamePortalDataSource_GetRowCount` | MiniGamePortalDataSource | Lazily populated row count for BaseClient.MiniGamePortal. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c47d90` | `MiniGamePortalDataSource_ScalarDeletingDtor` | MiniGamePortalDataSource | Scalar deleting destructor of the BaseClient.MiniGamePortal data source. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c481f0` | `EntryArray_FindByKey_Stride0x20` |  | Find one boss row by boss id in the MiniGameBossDataSource so the update/delete handlers can modify it (caller 00c48bd0). | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c48700` | `MiniGameBossDataSource_RemoveAllEntries` | MiniGameBossDataSource | Clears the whole 'MiniGame.Bosses' row set in one step, e.g. on leaving the boss view or on a reset message. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c48930` | `MiniGameBossDataSource_AgeEntries` | MiniGameBossDataSource | Periodic ageing/expiry pass over the boss rows (counter accumulates a caller-supplied amount until the threshold retires the row). | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c489b0` | `MiniGameBossDataSource_RemoveEntry` | MiniGameBossDataSource | Remove-by-key path used by the boss delete message handler 00c48c90. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c48bd0` | `MiniGameBossDataSource_AddOrUpdateEntry` | MiniGameBossDataSource | Applies a keyed update to the MiniGame.Bosses data-source entry array; the insert path's +0x18 value differs from the update path and remains unexplained. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c48ea0` | `MiniGameBossDataSource_RowCount` | MiniGameBossDataSource | Row-count method of the data source interface (vtable slot 2), telling the UI how many boss rows exist. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |
| `00c48eb0` | `MiniGameBossDataSource::~MiniGameBossDataSource` | MiniGameBossDataSource | Non-deleting destructor of the 'MiniGame.Bosses' data source. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |

## 🟡 speculative (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a69980` | `PiratesShipSelection_SelectAndConfirm ?` |  | Stores the chosen index locally, finds that ship entry, sends the selected names through the packet helper, and hides the ship-selection UI. | [[Native Map/Shards/S0303 00a674c0-00a6c3b0|S0303]] |
| `00ad3ae0` | `CaptureTcgPointer ? ⚠rename` |  | Shared virtual implementation for both card manager classes; changes pointer capture on the TCG window. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00c45fc0` | `IndexList_RebuildFromRecordArray ? ⚠downgrade` |  | Refresh the cached ordering of goal rows (flagged rows first) before the table reads values. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |

## ⚪ unknown (10)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009b7170` | `— ⚠reject` |  | Presents minigame terminal results; this code does not establish semantic names for the grouping byte or status values. | [[Native Map/Shards/S0271 009b2210-009b7170|S0271]] |
| `009b9fc0` | `— ⚠reject` |  | Apply a decoded mini-game group definition to the client manager and push every field to the mini-game group UI / data binding. | [[Native Map/Shards/S0272 009b7660-009bd590|S0272]] |
| `00a6a4b0` | `— ⚠reject` | ShipCombatProcessor | Drives every Pirates HUD screen (standings, no-symbol indicator, result screen and badges) from the current match state; called from the game-state machine 00a6b0c0 on every state change and update. | [[Native Map/Shards/S0303 00a674c0-00a6c3b0|S0303]] |
| `00ad3a60` | `— ⚠reject` |  | Shared virtual implementation for both card manager classes; operates on the named TCG window control. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00c08480` | `— ⚠reject` |  | Handler registered as 'SendMessage' in the MiniGameFlashC module (0x0182a1e8): forwards a message from the flash minigame host into the native side. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c463a0` | `— ⚠reject` | MiniGameGroupDataSource | Serves the columns of the BaseClient.MiniGameGroup catalogue rows used by the minigame browser UI. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c470b0` | `— ⚠reject` | MiniGameGroupDataSource | Tells the table how many minigame-group rows exist for BaseClient.MiniGameGroup. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c47220` | `— ⚠reject` | MiniGameAchievementsDataSource | Build one achievements data source instance; the renaming sibling 00c47680 registers BaseClient.MiniGame.Trophies while this variant is used by the base/minigame construction path. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c47c90` | `— ⚠reject` | MiniGamePortalDataSource | Shared construction step of the portal data source used by the registered constructor 00c47d00. | [[Native Map/Shards/S0392 00c42400-00c482e0|S0392]] |
| `00c48f40` | `— ⚠reject` | MiniGameBossDataSource | Polymorphic deletion entry for MiniGameBossDataSource held by its third base subobject vtable. | [[Native Map/Shards/S0393 00c48300-00c4e490|S0393]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00a69870` | `PiratesShipSelection_ConfirmSelection` | send | PacketTunneledClientPacket |  | Hypothesis: the original server endpoint received the tunneled SelectShip request containing the two length-prefixed names. The client establishes the emitted strings and UI hide action, but not server lookup, validation, or spawning behavior. |
| `00a69980` | `PiratesShipSelection_SelectAndConfirm ?` | send | PacketTunneledClientPacket |  |  |
