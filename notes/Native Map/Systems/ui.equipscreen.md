# System: ui.equipscreen

4 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bdaf80` | `EquipScreen_FinishedLoading` |  | Native trigger for the Lua equip-screen handler callback HandlerEquipScreen:FinishedLoading, called from 00bdb400 when the job change completes. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bdb400` | `EquipScreen_SelectJobAndFinish ?` |  | Selects the requested or current job in the equip-screen handler, then calls FinishedLoading only when 008c50f0 returns nonzero. The meaning of that predicate is unresolved. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bdb0a0` | `— ⚠reject` |  | Native trigger for HandlerEquipScreen:SelectJob, called from 00bdb400 with the job/profile id. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdb250` | `— ⚠reject` |  | Native trigger for HandlerEquipScreen:ErrorMessage; no caller is recorded in the map, so it is likely referenced from a function-pointer or script table. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |

