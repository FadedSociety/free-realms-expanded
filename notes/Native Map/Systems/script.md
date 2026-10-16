# System: script

5 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a1a7d0` | `InventoryHandler::OnIsItemUsable6` |  | Calls the inventory usability UI/Lua handler with six supplied arguments. | [[Native Map/Shards/S0289 00a18a90-00a1dd30|S0289]] |

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a4e130` | `Script_InvokeNamedMethod` |  | Calls a named script method (the FUN_009797a0 bridge parses 'Object:Method' names and runs them); the fixed 256-char buffer suggests it is used for short UI event names. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |

## 🟡 speculative (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00903b40` | `LuaCallNameWithIntArg ?` |  | Stack args param_1 (Lua function name, passed to FUN_009797a0) and param_2 (integer). Builds a one-entry Lua argument array with FUN_0070ec40, stores tag 1 and param_2 in the entry, calls FUN_009797a0(param_1, array, 0)… | [[Native Map/Shards/S0237 008ff3d0-00904e70|S0237]] |
| `00904060` | `LuaCallNameWithIntArg ?` |  | Stack args param_1 (Lua function name) and param_2 (integer). Byte-for-byte the same body as 00903b40 apart from its SEH handler address: builds a one-entry Lua argument array with FUN_0070ec40, stores tag 1 and param_2… | [[Native Map/Shards/S0237 008ff3d0-00904e70|S0237]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a1a4b0` | `— ⚠reject` |  | Calls the inventory usability UI/Lua handler with five supplied arguments. | [[Native Map/Shards/S0289 00a18a90-00a1dd30|S0289]] |

