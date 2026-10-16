# System: ui.pirates

8 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a600c0` | `PiratesStandings::SetPlayerFlag` | PiratesStandings | Updates the Pirates standings UI with three supplied flag values. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a60c10` | `PiratesShipSelection::PopulateSlots` | PiratesShipSelection | Populates the Pirates ship-selection movie's ship and decal slot ranges. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |

## 🟢 likely (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a612d0` | `DecrementCursor374AndRefresh` |  | Refreshes the ship-slot page after a bounded cursor decrement. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a612f0` | `IncrementCursor374AndRefresh` |  | Moves the visible ship-slot page forward within the list bounds. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a61320` | `DecrementCursor378AndRefresh` |  | Refreshes the decal-slot page after a bounded cursor decrement. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a61340` | `IncrementCursor378AndRefresh` |  | Moves the visible decal-slot page forward within the list bounds. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a63930` | `PiratesIndicators_Update` |  | HUD indicator and debug readout for the Pirates treasure/dock objectives; invoked from the Pirates processor 00a6a4b0. | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a62fd0` | `— ⚠reject` |  | Refresh of the Pirates scoreboard/standings UI; driven by the RefreshUI packet handler 00a655a0, which also carries the 'SwapPositions'/'SetPlayerFlag' siblings in 00a600c0. | [[Native Map/Shards/S0302 00a62280-00a67470|S0302]] |

