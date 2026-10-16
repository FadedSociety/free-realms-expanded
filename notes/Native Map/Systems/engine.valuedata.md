# System: engine.valuedata

10 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0127a480` | `ValueData::GetClassName` | ValueData | Class-name tag of the ValueData type. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `0127a4c0` | `ValueData::ScalarDeletingDtor` | ValueData | Heap-delete path of the ValueData object. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |

## 🟡 speculative (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0127a100` | `ParamListAddString_0127a100 ?` |  | Three-argument helper: calls FUN_012fd4b0(param_3) to build a temporary from the C string argument, then FUN_01312760(param_2, temp) to add it under the integer key param_2; calls FUN_01300cd0 with the stack cookie. Cal… | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `0127a170` | `ParamListAddInt_0127a170 ?` |  | Three-argument helper: calls FUN_012fa9f0(param_3) to build a temporary from the int argument, then FUN_01312760(param_2, temp) to add it under the integer key param_2; calls FUN_01300cd0 with the stack cookie. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `0127a490` | `ValueData::GetGlobal_01cbda60 ?` | ValueData | Returns the 32-bit global DAT_01cbda60 (6 bytes: mov eax, [01cbda60]; ret). Virtual slot [12] of ValueData::vftable. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `0127a4a0` | `ValueData::SetGlobal_01cbda60 ?` | ValueData | Stores its single stack argument into the global DAT_01cbda60 and returns, popping 4 bytes (12 bytes: mov eax, [esp+4]; mov [01cbda60], eax; ret 4). Virtual slot [13] of ValueData::vftable. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `0127ac50` | `Wrapper_0127ac50 ?` |  | Calls FUN_01300680 with the constant 6, then calls FUN_005f1e5c(argument) on the object at this+8 (thiscall with ecx = [this+8]); pops 4 bytes. 27 bytes, decoded from raw bytes. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |

## ⚪ unknown (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0127a1e0` | `— ⚠reject` |  | Three-argument helper: calls FUN_012fa980(param_3) to build a temporary from the third argument, then FUN_01312760(param_2, temp) to add it under the integer key param_2; calls FUN_01300cd0 with the stack cookie. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `0127a3e0` | `— ⚠reject` | ValueData | Constructs a ValueData cell with type tag 1 (boolean in the ValueData.h assertions) and zero value. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `0127a500` | `— ⚠reject` |  | Constructs a ValueData at stack address stack+0xc with FUN_0127a3e0, then calls FUN_01312760(param_2, local buffer) to add it under the integer key param_2; calls FUN_01300cd0 with the stack cookie. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |

