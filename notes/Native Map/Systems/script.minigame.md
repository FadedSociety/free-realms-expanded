# System: script.minigame

3 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009aae60` | `MiniGameGroup_SetPreselectedGameId` |  | Setter for the preselected minigame id in the client property/Lua space (name string is the anchor); called from 00bb4e40/00bb4f50 in the minigame start flow. | [[Native Map/Shards/S0269 009a6cf0-009aceb0|S0269]] |
| `009b6490` | `EmitUnlockUiEvent` |  | Emits the script event after releasing the temporary tagged Lua argument; it does not process external callback states. | [[Native Map/Shards/S0271 009b2210-009b7170|S0271]] |
| `009b6590` | `EmitMiniGameFlashKeyPressed` |  | Forwards native keyboard input to the mini-game Flash script. | [[Native Map/Shards/S0271 009b2210-009b7170|S0271]] |

