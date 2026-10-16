# System: ui.soccer

16 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (9)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b7c520` | `UpdateSoccerPossessionUi` |  | Refreshes the soccer scoreboard possession indicators from tracked entries. | [[Native Map/Shards/S0355 00b7c520-00b807c0|S0355]] |
| `00b7c6b0` | `UpdateSoccerArrows` |  | Maintains the soccer direction arrow and related UI state for the selected object. | [[Native Map/Shards/S0355 00b7c520-00b807c0|S0355]] |
| `00b7d210` | `UpdateSoccerArrowEntries` |  | Updates soccer arrow UI entries for tracked players and related ball state. | [[Native Map/Shards/S0355 00b7c520-00b807c0|S0355]] |
| `00b7de60` | `ApplySoccerUiState` |  | Keeps soccer panels synchronized with state transitions in the processor object. | [[Native Map/Shards/S0355 00b7c520-00b807c0|S0355]] |
| `00cd75b0` | `SoccerTeamStatsDataSource::~SoccerTeamStatsDataSource` | SoccerTeamStatsDataSource | Releases the team's soccer statistics data source. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd7620` | `SoccerPlayerStatsDataSource::~SoccerPlayerStatsDataSource` | SoccerPlayerStatsDataSource | Releases the player statistics data source. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd7690` | `SoccerPlayerActionsDataSource::~SoccerPlayerActionsDataSource` | SoccerPlayerActionsDataSource | Releases the player actions data source. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd7700` | `SoccerTeamStatsDataSource::SoccerTeamStatsDataSource` | SoccerTeamStatsDataSource | Creates the data source used by the team statistics view. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd77a0` | `SoccerPlayerStatsDataSource::SoccerPlayerStatsDataSource` | SoccerPlayerStatsDataSource | Creates the data source used by the player statistics view. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |

## 🟢 likely (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b77220` | `PositionUiElement` |  | Positions/animates a soccer HUD element (charge bar or ball marker) with a world-space anchor and a scale; called from FUN_00b7a4a0 and FUN_00b7e910. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00cdac00` | `SoccerPlayer::UpdateOverheadText` | SoccerPlayer | Updates the overhead text element during SoccerPlayer's update path. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cdae70` | `SoccerPlayer::UpdateOverlayElementState` | SoccerPlayer | Updates an overlay element during SoccerPlayer cleanup/state handling. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |

## 🟡 speculative (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cd7780` | `SoccerTeamStatsDataSource::DeletingDestructor ?` | SoccerTeamStatsDataSource | Implements the scalar deleting destructor for the team statistics data source. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd7820` | `SoccerPlayerStatsDataSource::DeletingDestructor ?` | SoccerPlayerStatsDataSource | Implements the scalar deleting destructor for the player statistics data source. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |
| `00cd78e0` | `SoccerPlayerActionsDataSource::DeletingDestructor ?` | SoccerPlayerActionsDataSource | Implements the scalar deleting destructor for the player actions data source. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cd7840` | `— ⚠reject` | SoccerPlayerActionsDataSource | Creates and initializes the soccer player actions table data source. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |

