# System: game.objective

7 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009aaf50` | `ThunkField2bea4_00c43140` |  | Objective-event service thunk: the vault note Research/Original Opening Microtutorial Recovery.md binds the Lua method Ui.OnObjectiveEvent to 00c1f0a0, which reaches 009aaf50 -> 00c43140, the dispatch to enabled, unfini… | [[Native Map/Shards/S0269 009a6cf0-009aceb0|S0269]] |
| `009aaf60` | `GatedCall_00c43180` |  | Conditional forwarder in the same +0x2bea4 service group as FUN_009aaf50/FUN_009aaf80 (target 00c43180, the sibling of the objective-event dispatcher 00c43140); the gate reads a flag word inside the object reached throu… | [[Native Map/Shards/S0269 009a6cf0-009aceb0|S0269]] |
| `009aaf80` | `ThunkField2bea4_00c431c0` |  | Sibling of the objective-event service thunk FUN_009aaf50 in the same +0x2bea4 group (target 00c431c0, the function next to the objective-event dispatcher 00c43140); callers 00ab0d60/00ab1e30/00ab2250. | [[Native Map/Shards/S0269 009a6cf0-009aceb0|S0269]] |
| `009ad510` | `UpdateObjectiveRecordStatus` |  | Updates a record in the minigame objective flow. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009ad5a0` | `SetObjectiveRecordField44` |  | Updates an objective record in the same minigame objective flow. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009ad620` | `CountIncompleteObjectives` |  | End-score handler uses this to summarize objective rows. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |
| `009ad6a0` | `CountObjectiveRowsWithFlag28` |  | Called from end-score handling. | [[Native Map/Shards/S0270 009acf20-009b2140|S0270]] |

