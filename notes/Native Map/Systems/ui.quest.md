# System: ui.quest

56 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (18)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a10870` | `GetAutoSwitchSelectedQuest` |  | Getter for the 'auto-switch selected quest' toggle; Research/Native Quest Journal UI.md cites this address as the getter reached from the Lua binding 00c0a260. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a10880` | `AutoSwitchSelectedQuest_Set` |  | Setter path of the AutoSwitch-selected-quest flag: Research/Native Quest Journal UI.md cites this address for the Lua binding SetAutoSwitchSelectedQuest -> 00c0a200 -> 00a10880 -> 00c7c5d0. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a8f130` | `QuestHandlerUpdateObjective` |  | Refreshes quest objective presentation through the QuestHandler Lua method. | [[Native Map/Shards/S0310 00a8ce60-00a91b00|S0310]] |
| `00a8f2b0` | `QuestHandlerPopulateHelper` |  | Refreshes the quest helper through the QuestHandler Lua method. | [[Native Map/Shards/S0310 00a8ce60-00a91b00|S0310]] |
| `00a92680` | `NotifyQuestStarted` |  | Presents the client-side quest-start notification; it does not parse a packet or establish server quest policy. | [[Native Map/Shards/S0311 00a91bf0-00a97400|S0311]] |
| `00a928b0` | `NotifyQuestCompleted` |  | Displays a client-side quest-completion notification; this body does not prove who authorized completion. | [[Native Map/Shards/S0311 00a91bf0-00a97400|S0311]] |
| `00a92ac0` | `NotifyQuestFailed` |  | Displays a client-side quest-failure notification; failure policy is not present in this callback. | [[Native Map/Shards/S0311 00a91bf0-00a97400|S0311]] |
| `00a92cd0` | `NotifyQuestAbandoned` |  | Displays abandoned-quest text through a shared FailQuest event; the key/event pairing is explicit in this body. | [[Native Map/Shards/S0311 00a91bf0-00a97400|S0311]] |
| `00a92ee0` | `NotifyNewQuestObjective` |  | Prepares the client-side new-objective notification from quest and objective strings. | [[Native Map/Shards/S0311 00a91bf0-00a97400|S0311]] |
| `00a931b0` | `NotifyQuestObjectiveUpdated` |  | Native quest-update callback used by the journal/helper path; its route is documented in Research/Native Quest Journal UI.md. | [[Native Map/Shards/S0311 00a91bf0-00a97400|S0311]] |
| `00a936e0` | `NotifyQuestObjectiveCompleted` |  | Native objective-completion presentation callback, separate from authoritative completion checks. | [[Native Map/Shards/S0311 00a91bf0-00a97400|S0311]] |
| `00a939b0` | `NotifyQuestObjectiveFailed` |  | Displays an objective-failure message; failure policy is not present in this UI callback. | [[Native Map/Shards/S0311 00a91bf0-00a97400|S0311]] |
| `00a94be0` | `ShowQuestStartScreen` |  | Opens the client quest-start presentation and can show NPC intro text; reward/server policy is outside this body. | [[Native Map/Shards/S0311 00a91bf0-00a97400|S0311]] |
| `00a95420` | `ShowQuestEndScreen` |  | Opens the client quest-end presentation; the additional helper call depends on param_6 and a resolved NPC record. | [[Native Map/Shards/S0311 00a91bf0-00a97400|S0311]] |
| `00c0a020` | `RegisterQuestScreenModule` |  | Registers the client 'QuestScreen' Lua module (AcceptQuest, DeclineQuest, SelectQuest, AbandonQuest, SelectQuestInHelper, EndQuest, GetSelectedQuest, auto-switch toggles, GetQuestLimit, SelectMiscQuestFromBook). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00cf7a10` | `QuestDataSource::GetColumnName` | QuestDataSource | Provides column names for the quest journal datasource. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf7ae0` | `QuestObjectiveDataSource::GetColumnName` | QuestObjectiveDataSource | Provides column names for objective rows in the quest journal. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf81f0` | `QuestDataSource::QuestDataSource` | QuestDataSource | Constructs the quest journal datasource. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |

## 🟢 likely (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a10800` | `AutoSwitchQuestSetter_Enable` |  | One of the two AutoSwitch-selected-quest setters (enable path) reached from the Lua binding layer; the sub-object at +0x1add0 is the quest-journal region named in Research/Native Quest Journal UI.md. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a10820` | `MethodThunk_00c7c540` |  | Quest-journal sub-object member thunk (+0x1add0). | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a10830` | `Quest_SelectMiscQuestFromBookThunk` |  | Wrapper reached by Quest_SelectMiscQuestFromBook; the constant distinguishes this path from the adjacent constant-1 wrapper. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00c0a080` | `Quest_SelectQuest` |  | Handler registered as 'SelectQuest' in the QuestScreen module (0x0182a6f8). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0a140` | `Quest_SelectMiscQuestFromBook` |  | Handler registered as 'SelectMiscQuestFromBook' in the QuestScreen module (0x0182a6f8). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0a500` | `Quest_AbandonQuest` |  | Handler registered as 'AbandonQuest' in the QuestScreen module (0x0182a6f8): drops the selected quest. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |

## 🟡 speculative (25)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ce7ea0` | `ProgressiveQuestDataSource::ColumnName ?` | ProgressiveQuestDataSource | Maps column indices 0..11 to progressive quest id/name/timer/slot/icon/objective/window/button fields; returns <error> otherwise. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce7f50` | `ProgressiveQuestDataSource::GetHasQuest ? ⚠rename` | ProgressiveQuestDataSource | Returns whether the quest data pointer at +0x228 has a nonzero field at offset +0x10. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce7f70` | `ProgressiveQuestSlotsDataSource::ColumnName ?` | ProgressiveQuestSlotsDataSource | Maps column indices 0..11 to slot/item/key/quest eligibility, key bundle, claim, tooltip, name, and quest id fields. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce8020` | `ProgressiveQuestPrizeSlotsDataSource::ColumnName ?` | ProgressiveQuestPrizeSlotsDataSource | Maps column indices 0..6 to prize slot, icon, progress, claim eligibility, tooltip, name, and quest id fields. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce80b0` | `ProgressiveQuestSlotsDataSource::GetField38 ? → ProgressiveQuestSlotsDataSource…` | ProgressiveQuestSlotsDataSource | Reads nested quest data pointer and returns its field at offset +0x38, or zero when pointer is null. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce80d0` | `ProgressiveQuestPrizeSlotsDataSource::GetField48 ? → ProgressiveQuestPrizeSlots…` | ProgressiveQuestPrizeSlotsDataSource | Reads nested quest data pointer and returns its field at offset +0x48, or zero when pointer is null. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce80f0` | `ProgressiveQuestSlotArrayAt ?` |  | Returns element address at base plus index times 0x24; for index beyond count, requests growth and stores new count. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce8460` | `ProgressiveQuestDataSource::GetColumn ?` | ProgressiveQuestDataSource | Reads the current quest record and returns quest id, localized name, timer, active slot, icon, objective strings, window flag, and button string ids for supported columns. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce89d0` | `ProgressiveQuestDataSource::Ctor ?` | ProgressiveQuestDataSource | Constructs ProgressiveQuest.Definition holder, installs source vtables, and stores two supplied arguments. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce8ab0` | `ProgressiveQuestDataSource::ScalarDeletingDtor ?` | ProgressiveQuestDataSource | Calls the embedded holder destructor, conditionally frees the object when flag bit 0 is set, and returns its pointer. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce8b10` | `ProgressiveQuestSlotsDataSource::Ctor ?` | ProgressiveQuestSlotsDataSource | Constructs ProgressiveQuest.Slots holder, installs slots source vtables, and stores two supplied arguments. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce8bf0` | `ProgressiveQuestSlotsDataSource::ScalarDeletingDtor ?` | ProgressiveQuestSlotsDataSource | Calls the embedded holder destructor, conditionally frees the object when flag bit 0 is set, and returns its pointer. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce8c50` | `ProgressiveQuestPrizeSlotsDataSource::Ctor ?` | ProgressiveQuestPrizeSlotsDataSource | Constructs ProgressiveQuest.PrizeSlots holder, installs prize slots source vtables, and stores two supplied arguments. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce8d30` | `ProgressiveQuestPrizeSlotsDataSource::ScalarDeletingDtor ?` | ProgressiveQuestPrizeSlotsDataSource | Calls the embedded holder destructor, conditionally frees the object when flag bit 0 is set, and returns its pointer. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00cf7ad0` | `QuestDatasourceResetThunk ?` |  | Quest datasource reset callback, as indicated by quest removal caller context. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf7bb0` | `QuestDataSource::GetRowCount ?` | QuestDataSource | Returns the row count for the quest datasource. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf7bc0` | `QuestObjectiveDataSource::GetRowCount ?` | QuestObjectiveDataSource | Returns the row count for the quest objective datasource. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf7c30` | `BuildQuestRows ?` |  | Builds rows for the quest journal datasource. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf7ca0` | `QuestDataSource::SelectQuest ?` | QuestDataSource | Applies a quest selection to the journal datasource. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf7cd0` | `QuestDataSource::GetCell ?` | QuestDataSource | Supplies quest journal cell values. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf8110` | `QuestDataSource::Cleanup ?` | QuestDataSource | Cleans up the quest datasource base and owned table. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf8180` | `QuestObjectiveDataSource::Cleanup ?` | QuestObjectiveDataSource | Cleans up the objective datasource base and owned table. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf8280` | `QuestDataSource::ScalarDeletingDtor ?` | QuestDataSource | Scalar deleting destructor for QuestDataSource. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf82a0` | `QuestObjectiveDataSource::QuestObjectiveDataSource ?` | QuestObjectiveDataSource | Constructs a quest objective datasource. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf8330` | `QuestObjectiveDataSource::ScalarDeletingDtor ?` | QuestObjectiveDataSource | Scalar deleting destructor for QuestObjectiveDataSource. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |

## ⚪ unknown (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c0a0e0` | `— ⚠reject` |  | Handler registered as 'SelectQuestInHelper' in the QuestScreen module (0x0182a6f8): selects the quest shown in the helper/tracker panel. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00ce81a0` | `— ⚠reject` |  | Returns element address at base plus index times 0x1c; for index beyond count, requests growth and stores new count. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce85f0` | `— ⚠reject` | ProgressiveQuestSlotsDataSource | Bounds-checks slot rows and returns slot id, icon, slot/key/quest predicates, key bundle values, progress, tooltip and localized labels. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ce8860` | `— ⚠reject` | ProgressiveQuestPrizeSlotsDataSource | Bounds-checks prize rows and returns prize slot ids, icon, progress, claim predicate, tooltip/name strings, and quest id. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00cf7be0` | `— ⚠reject` |  | Builds objective datasource rows for a selected quest. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf7c70` | `— ⚠reject` | QuestDataSource | Refreshes quest datasource content. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf7fd0` | `— ⚠reject` | QuestObjectiveDataSource | Supplies objective cell values in the quest journal. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |

