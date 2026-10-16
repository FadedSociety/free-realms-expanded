# System: ui.player_positions

19 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a85900` | `PlayerPositionsDataSource::GetColumnName` | PlayerPositionsDataSource | Supplies the column titles of the BaseClient.PlayerPositions table (the friends/group position list). | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a85a70` | `PlayerPositionsDataSource::GetRowCount` | PlayerPositionsDataSource | Number of rows in the BaseClient.PlayerPositions table. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a86050` | `PlayerPositionsDataSourceCtor` | PlayerPositionsDataSource | Creates the BaseClient.PlayerPositions table fed by the friends/group position replies. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a860f0` | `PlayerPositionsDataSourceDtor` | PlayerPositionsDataSource | Tears down BaseClient.PlayerPositions and its row list. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a86170` | `PlayerPositionsDataSourceScalarDeletingDtor` | PlayerPositionsDataSource | Vtable slot 0 of the positions data source. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |

## 🟢 likely (13)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009d1d20` | `NamePlatesInternals::NamePlateManagerCtor ⚠downgrade` |  | Initializes name-plate management state and its named lookup keys. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `009d1eb0` | `AllocateNamePlateManager` |  | Creates name-plate manager storage for client initialization. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `009d1f20` | `NamePlatesInternals::NamePlateManagerDtor` |  | Releases name-plate manager-owned lists and maps. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `00a85ae0` | `PlayerPositionDtorBody` | CommandPacketRequestPlayerPositionsRepl… | Releases the row's name string; reached from the list node removal (00a85d60) and from unwind funclets. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a85b50` | `EhCleanupCallPlayerPositionDtor` |  | Compiler-generated unwind cleanup wrapper for the PlayerPosition destructor. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a85ba0` | `PlayerPositionCtor` | CommandPacketRequestPlayerPositionsRepl… | Creates an empty row before the position/name/flag fields are filled from the parsed packet. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a85c20` | `PlayerPositionCtorWrapper` | CommandPacketRequestPlayerPositionsRepl… | Out-of-line construction entry used by the list append path. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a85c70` | `PlayerPositionScalarDeletingDtor` | CommandPacketRequestPlayerPositionsRepl… | Vtable-style invalidation when a row node is deleted. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a85cd0` | `PlayerPositionsListAppend` | SoeUtil::List | Adds one received player-position row to the data source list. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a85d60` | `PlayerPositionsListRemoveNode` | SoeUtil::List | Removal helper shared by the clear/refresh paths of the positions table. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a85df0` | `PlayerPositionsDataSourceAppendRow` | PlayerPositionsDataSource | Adds one player position (from a received position reply) to the BaseClient.PlayerPositions table. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a85f90` | `PlayerPositionListClear` | SoeUtil::List | Clears a position list (used before it is repopulated or destroyed). | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a85fb0` | `PlayerPositionsDataSourceClear` | PlayerPositionsDataSource | Clears BaseClient.PlayerPositions, e.g. when the position reply is about to be replaced by a fresh one. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a85e70` | `— ⚠reject` | PlayerPositionsDataSource | Reads one column (guid, name, X/Z location, friend/group/encounter flags) of one row of the friends/group position table. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |

