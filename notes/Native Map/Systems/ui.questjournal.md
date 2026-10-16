# System: ui.questjournal

15 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a432d0` | `HasHubQuestDefinition` |  | Predicate that decides whether a quest id has a hub definition, i.e. whether it belongs in the chapter book rather than Other Quests; the vault note records that this checks the 32-bucket hub-quest definition map and no… | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00a44bb0` | `ShowAdvJournalIfHubQuest` |  | Opens the Adventure Journal for a quest id that has a hub definition; the plain string plus the accepted caller label (00c7cd40 is the quest add/remove dispatch) anchor the feature. | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |

## 🟢 likely (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a43310` | `InvokeSlot10OnField34` |  | Quest-removal notification path: the accepted caller label records that 00c7ab20 is the removal bookkeeping that notifies the journal datasource; this helper is the notification half. | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00a43c90` | `JournalData_Ctor` |  | Builds the client-side journal state object and its four data sources; the constructor is called from the client constructor FUN_0092b170, and the four data-source classes are identified by their own registry strings (R… | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00a43f00` | `AnyListEntryKeyInMap24` |  | Predicate that answers whether the definition entry selected by param_2 is already recorded in the map at this+0x24, used by the region/hub selection logic in 00a43f80 and by the data source getter 00c98170. | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00a43f60` | `MaxNonnegativeListNodeField0` |  | Caller 00a447d0 uses the result as a bound in the journal-selection rebuild. | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00a44020` | `RecomputeCurrentRegionId` |  | Keeps the journal's current region/hub id in step with the recorded-map after any change; called by the selection setters 00a44790/00a447d0 and by the handler 00a45d80 after loading definitions. | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00a44100` | `HasSelectableEntryForCategory` |  | Answers whether a category has anything to show, which drives the fallback choice inside the selection rebuild 00a447d0 and the data source getter 00c97f30. | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00a44790` | `SelectChapterAndRefresh` |  | Selection setter of the journal; the same notification shape appears in 00a447d0 and 00a43310, and it is the path the journal UI uses when the player picks a different chapter. | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00a447d0` | `RebuildJournalSelection` |  | Central refresh of the journal book after any change (chapter added, removed or selected): 00a44af0/00a44b10/00a44b30/00a44b70/00a44bb0 all funnel into it, and it is what makes the data sources show the new selection. | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00a44af0` | `SetField8AndRebuild` |  | Setter used to pin an explicit chapter id; the caller list is empty, so it is reached indirectly (the same +8 field is read as the explicit chapter by 00a447d0). | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00a44b10` | `TrackSelectedIdAndRebuild` |  | Ref-counted selection helper: the counter lets the same selection be requested by several owners while the id stays pinned (00a44b30 is the matching decrement). | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00a44b30` | `UntrackSelectedIdAndRebuild` |  | Counterpart of 00a44b10, called from 00ceb8d0 when a selection owner goes away. | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |
| `00a45370` | `JournalDefinitionMaps_TeardownB` |  | Destructor of the journal definition holder reached from 00a45a40 and the journal packet handler 00a45d80; the four maps it destroys are the ones the sub-opcode 1 dictionary fills. | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a44b70` | `SelectEntryByField14AndRebuild ?` |  | Called by 008a78d0 and 00a45d80 in the journal selection path. The meaning of node+0x14 and the stored node id is not established. | [[Native Map/Shards/S0296 00a3f560-00a45650|S0296]] |

