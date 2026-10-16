# System: engine.ui

2 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a81110` | `ValueCellAssignPtr_00a81110` |  | Row-value writers of the matchmaking columns use this copy to put a value into the UI's output cell. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a7bfc0` | `— ⚠reject` |  | Small shared helper the row-value getters in this file use to write a value into the UI's output cell; the same code at 009969b0 is used by the rest of the UI layer. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |

