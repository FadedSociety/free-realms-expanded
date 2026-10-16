# System: ui.lobby

33 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (17)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a224a0` | `EncounterListDataSource::ColumnName` | LobbyDataSourceManager::EncounterListDa… | Virtual slot 1 of the lobby encounter-list data source: names the columns the lobby UI shows for each running encounter/game. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |
| `00a23940` | `EncounterListDataSource::RefreshRows` | LobbyDataSourceManager::EncounterListDa… | Virtual slot 14 of the lobby encounter-list data source: rebuilds the visible encounter rows for the lobby UI. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |
| `00a24a60` | `LobbyDataSourceManager::GameListDataSource::GetColumnValue` | LobbyDataSourceManager::GameListDataSou… | Returns one game-list column value to the lobby UI. | [[Native Map/Shards/S0291 00a244a0-00a29500|S0291]] |
| `00a25a40` | `LobbyDataSourceManager::EncounterListDataSource::GetItemColumn` | LobbyDataSourceManager::EncounterListDa… | Supplies values for a lobby encounter-list row. | [[Native Map/Shards/S0291 00a244a0-00a29500|S0291]] |
| `00a26f30` | `LobbyDataSourceManager::GameListDataSource::Dtor` | LobbyDataSourceManager::GameListDataSou… | Destroys the lobby game-list data source. | [[Native Map/Shards/S0291 00a244a0-00a29500|S0291]] |
| `00a26fd0` | `LobbyDataSourceManager::EncounterListDataSource::Dtor` | LobbyDataSourceManager::EncounterListDa… | Destroys the lobby encounter-list data source. | [[Native Map/Shards/S0291 00a244a0-00a29500|S0291]] |
| `00a27190` | `LobbyDataSourceManager::GameListDataSource::Ctor` | LobbyDataSourceManager::GameListDataSou… | Constructs the lobby game-list UI data source. | [[Native Map/Shards/S0291 00a244a0-00a29500|S0291]] |
| `00a27230` | `LobbyDataSourceManager::GameListDataSource::ScalarDeletingDtor` | LobbyDataSourceManager::GameListDataSou… | Scalar deleting destructor for game-list data source. | [[Native Map/Shards/S0291 00a244a0-00a29500|S0291]] |
| `00a27250` | `LobbyDataSourceManager::EncounterListDataSource::Ctor` | LobbyDataSourceManager::EncounterListDa… | Constructs lobby encounter-list UI data source. | [[Native Map/Shards/S0291 00a244a0-00a29500|S0291]] |
| `00a272f0` | `LobbyDataSourceManager::EncounterListDataSource::ScalarDeletingDtor` | LobbyDataSourceManager::EncounterListDa… | Scalar deleting destructor for encounter-list data source. | [[Native Map/Shards/S0291 00a244a0-00a29500|S0291]] |
| `00c91120` | `GameConfigOptionDataSource::ColumnName` | LobbyDataSourceManager::GameConfigOptio… | Expose lobby game configuration option fields to the UI. | [[Native Map/Shards/S0406 00c8f830-00c95560|S0406]] |
| `00c911c0` | `GameConfigOptionDataSource::CountValues` | LobbyDataSourceManager::GameConfigOptio… | Report how many game configuration option rows the UI can display. | [[Native Map/Shards/S0406 00c8f830-00c95560|S0406]] |
| `00c91310` | `GameConfigOptionDataSource::GetColumnValue` | LobbyDataSourceManager::GameConfigOptio… | Supply lobby game configuration rows to the UI data source. | [[Native Map/Shards/S0406 00c8f830-00c95560|S0406]] |
| `00c914b0` | `GameConfigOptionDataSource::Dtor` | LobbyDataSourceManager::GameConfigOptio… | Destroy lobby option data-source state and owned strings. | [[Native Map/Shards/S0406 00c8f830-00c95560|S0406]] |
| `00c91580` | `GameConfigOptionDataSource::Ctor` | LobbyDataSourceManager::GameConfigOptio… | Initialize a lobby game configuration option data source. | [[Native Map/Shards/S0406 00c8f830-00c95560|S0406]] |
| `00c91650` | `GameConfigOptionDataSource::ScalarDeletingDtor` | LobbyDataSourceManager::GameConfigOptio… | Implement scalar-deleting destruction for this data-source class. | [[Native Map/Shards/S0406 00c8f830-00c95560|S0406]] |
| `00c91670` | `PopulateLobbyGameOptions` |  | Populate the lobby configuration option data sources presented to the UI. | [[Native Map/Shards/S0406 00c8f830-00c95560|S0406]] |

## 🟢 likely (11)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a223d0` | `GameListDataSource::CountPlus3` | LobbyDataSourceManager::GameListDataSou… | Virtual slot 0 of the lobby game-list data source: supplies a small integer derived from the current game record to the lobby UI. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |
| `00a22470` | `GameListDataSource::ClearFlagAndForward` | LobbyDataSourceManager::GameListDataSou… | Virtual slot 4 of the lobby game-list data source: clears the data source's populated flag and forwards to the shared data-source helper. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |
| `00a22560` | `EncounterListDataSource::ClearFlagAndForward` | LobbyDataSourceManager::EncounterListDa… | Virtual slot 4 of the lobby encounter-list data source: clears its populated flag and forwards to the shared helper. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |
| `00a25870` | `LobbyDataSourceManager::GameListDataSource::GetItemColumn` | LobbyDataSourceManager::GameListDataSou… | Supplies values for a lobby game-list row. | [[Native Map/Shards/S0291 00a244a0-00a29500|S0291]] |
| `00a29370` | `LobbyDataSourceManager::Dtor` |  | Destroys lobby game and encounter list registrations. | [[Native Map/Shards/S0291 00a244a0-00a29500|S0291]] |
| `00a29500` | `LobbyDataSourceManager::Ctor` |  | Constructs lobby UI list manager. | [[Native Map/Shards/S0291 00a244a0-00a29500|S0291]] |
| `00c15ac0` | `JoinLobbyGame` |  | Joins the lobby game identified by the script argument; the name is the table row 0x0182e270 of the client's global script table, and the lobby packets themselves are built outside this shard. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `0102a040` | `CollectDisplayOrderPairs10` |  | Collects the display-order pairs for one lobby option key into a temporary list (callers 0102a290, 0102a5a0, 0102a690 and 00c90f50). | [[Native Map/Shards/S0565 01025bf0-0102b850|S0565]] |
| `0102a160` | `CollectDisplayOrderValuePairs10` |  | Collects the display-order value pairs for one lobby option key (callers 00c91060 and 00c911c0). | [[Native Map/Shards/S0565 01025bf0-0102b850|S0565]] |
| `0102a5a0` | `ResolveKeyThenCollectPairs` |  | Resolves a lobby option key to its display-order entries (caller 00a223d0, GameListDataSource::CountPlus3 accepted label). | [[Native Map/Shards/S0565 01025bf0-0102b850|S0565]] |
| `0102a690` | `LookupCollectedPairValue` |  | Returns one display-order value for a lobby option key and an index (caller 00a24a60, GetColumnValue accepted confirmed). | [[Native Map/Shards/S0565 01025bf0-0102b850|S0565]] |

## 🟡 speculative (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a22400` | `GameListDataSource::ValueOrZero ?` | LobbyDataSourceManager::GameListDataSou… | Virtual slot 2 routine for GameListDataSource: conditionally calls slot +0x38 on the referenced object, then returns 010c8f30's value when the object chain is present or zero otherwise. The called slot and returned valu… | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |
| `00a22520` | `EncounterListDataSource::ValueOrZero ? ⚠downgrade` | LobbyDataSourceManager::EncounterListDa… | Virtual slot 2 of the lobby encounter-list data source: refresh-on-demand plus current-encounter value. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |
| `00a27820` | `MaybeRequestLobbyDefinitionsAfterDelay ?` |  | Triggers lobby definition request flow after a delay. | [[Native Map/Shards/S0291 00a244a0-00a29500|S0291]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a22fa0` | `— ⚠reject` | LobbyDataSourceManager::GameListDataSou… | Rebuilds rows through the LobbyDataSourceManager::GameListDataSource interface and sets its populated flag; the source list?s authority and comparison values? meaning are unknown. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |
| `0102a290` | `— ⚠reject` |  | Stores param_1 into *param_2, calls 0102a040 with param_1 and the address of a stack-local List<int,-1>, then walks the temporary list it was given: each id is looked up with 01028c50 and, when found, a two-dword pair b… | [[Native Map/Shards/S0565 01025bf0-0102b850|S0565]] |

