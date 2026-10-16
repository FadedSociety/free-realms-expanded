# System: ui.group

29 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (22)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a7c810` | `GroupMemberDataSource::GetColumnName` | GroupMemberDataSource | Names the columns the group member data source exposes to the UI/script layer. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7c960` | `GroupDataSource::GetColumnName` | GroupDataSource | Names the single column the group data source exposes to the UI/script layer. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7ca90` | `RaiseGroupMemberDataUpdatedEvent` |  | Refreshes the group datasource and notifies the registered group-screen handler when present. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7cbd0` | `RaiseGroupDataUpdatedEvent` |  | Refreshes the group datasource and notifies the registered group-screen handler when present. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7cd60` | `GroupMemberDataSource::GetColumnValue` | GroupMemberDataSource | Supplies every column the group member list displays; the column names are published by GroupMemberDataSource::GetColumnName (00a7c810). | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7d3b0` | `RaiseSetGroupMemberSelectingJob` |  | Maps the supplied group-member guid pair to a row for the minigame-start script event; the event uses row 0 when the pair matches the cached owner pair or no scan row matches. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7d660` | `RaiseGroupMemberRowDataUpdatedEvent` |  | Refreshes one group-member row and notifies the group-screen handler only when that handler is registered. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7d830` | `RaiseActivateOrDeactivateEncounterRequest` |  | Routes activation/deactivation encounter-request events to the registered group-screen script handler; the body does not establish a dialogue or other UI presentation. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7db00` | `GroupMemberDataSource::~GroupMemberDataSource` | GroupMemberDataSource | Tears down the group member data source object. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7db70` | `GroupDataSource::~GroupDataSource` | GroupDataSource | Tears down the group (single row) data source object. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7dbe0` | `GroupMemberDataSource::GroupMemberDataSource` | GroupMemberDataSource | Creates the data source the group member list binds to. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7dc60` | `GroupMemberDataSource::ScalarDeletingDtor` | GroupMemberDataSource | Standard MSVC deleting destructor for the group member data source. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7dc80` | `GroupDataSource::GroupDataSource` | GroupDataSource | Creates the single-row data source the group panel binds to. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7dd00` | `GroupDataSource::ScalarDeletingDtor` | GroupDataSource | Standard MSVC deleting destructor for the group data source. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a81130` | `FindGuildMemberDataSourceByName` |  | Resolves the group screen's GuildMemberDataSource by its registered name. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a899f0` | `CombatGroupHandlerEnsureOpen` |  | Initialises and opens the combat group UI handler once per object, when the combat gate passes. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a89a40` | `CombatGroupHandlerClose` |  | Closes the combat group UI handler when it is open. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a89a70` | `VehicleRaceGroupHandlerClose` |  | Closes the vehicle-race group UI handler when it is open. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a89aa0` | `SoccerGroupHandlerEnsureOpen` |  | Initialises and opens the soccer group UI handler once per object, when the soccer gate passes. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a89b00` | `SoccerGroupHandlerClose` |  | Closes the soccer group UI handler when it is open. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a89b30` | `VehicleDemolitionDerbyGroupHandlerClose` |  | Closes the vehicle demolition derby group UI handler when it is open. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00c0b080` | `RegisterGroupScreenModule` |  | Registers the client 'GroupScreen' Lua module (DepartGroup, RemovePlayerFromGroup, RemovePlayerFromGroupByGuid, ActivateEncounter, SetChangingJob). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |

## 🟢 likely (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a7c980` | `GroupDataSource::GetRowCount` | GroupDataSource | Row-count query of the group panel's data source; the group list is a single row. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7c9d0` | `GroupMemberDataSource::GetRowCount` | GroupMemberDataSource | Row-count query of the group member list (one row per member, or a single placeholder row when no group exists). | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7c9f0` | `GroupMemberDataSource::RefreshMemberRows` | GroupMemberDataSource | Fills the group member list with the members other than the local player and marks the data source changed so the UI refreshes. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7cd10` | `GroupDataSource::GetColumnValue ⚠downgrade` | GroupDataSource | Answers the single MinigameType column of the group row, resolving the value from the current group through FUN_009ad290. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |
| `00a7da40` | `RefreshMemberRowForGuid` |  | Finds a member row for a guid pair before triggering the row refresh event; the notifier call is gated by the owner handle, and a failed scan leaves index 0. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c0b020` | `Group_RemovePlayerFromGroup ? ⚠downgrade` |  | Handler registered as 'RemovePlayerFromGroup' in the GroupScreen module (0x0182a988). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c0b300` | `— ⚠reject` |  | Handler registered as 'SetChangingJob' in the GroupScreen module (0x0182a988): tells the server the character is (or is no longer) changing jobs, which suppresses grouping. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |

