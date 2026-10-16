# System: crt.atexit

16 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0174f420` | `StaticDtorThunk_01cbddbc` |  | Static-object destructor thunk for the object at 01cbddbc: it hands that object to 012a4570, a cleanup routine that the map EH rule classes as a destructor (called from 14 unwind funclets). | [[Native Map/Shards/S0804 0174b4a0-0174f4f0|S0804]] |
| `0174f430` | `StaticDtorThunk_01cbdde0` |  | Static-object destructor thunk for the object at 01cbdde0: it hands that object to 012a4570, a cleanup routine that the map EH rule classes as a destructor (called from 14 unwind funclets). | [[Native Map/Shards/S0804 0174b4a0-0174f4f0|S0804]] |
| `0174f470` | `StaticDtorThunk_01cbdd88` |  | Static-object destructor thunk for the object at 01cbdd88: it hands that object to 012a4570, a cleanup routine that the map EH rule classes as a destructor (called from 14 unwind funclets). | [[Native Map/Shards/S0804 0174b4a0-0174f4f0|S0804]] |
| `0174f4c0` | `StaticDtorThunk_01cbeb90` |  | Static-object destructor thunk for the object at 01cbeb90: it hands that object to 012a4570, a cleanup routine that the map EH rule classes as a destructor (called from 14 unwind funclets). | [[Native Map/Shards/S0804 0174b4a0-0174f4f0|S0804]] |
| `0174f4d0` | `StaticDtorThunk_01cbeb54` |  | Static-object destructor thunk for the object at 01cbeb54: it hands that object to 012a4570, a cleanup routine that the map EH rule classes as a destructor (called from 14 unwind funclets). | [[Native Map/Shards/S0804 0174b4a0-0174f4f0|S0804]] |
| `0174f4e0` | `StaticDtorThunk_01cbeb60` |  | Static-object destructor thunk for the object at 01cbeb60: it hands that object to 012a4570, a cleanup routine that the map EH rule classes as a destructor (called from 14 unwind funclets). | [[Native Map/Shards/S0804 0174b4a0-0174f4f0|S0804]] |
| `0174f4f0` | `StaticDtorThunk_01cbeb6c` |  | Static-object destructor thunk for the object at 01cbeb6c: it hands that object to 012a4570, a cleanup routine that the map EH rule classes as a destructor (called from 14 unwind funclets). | [[Native Map/Shards/S0804 0174b4a0-0174f4f0|S0804]] |

## 🟡 speculative (9)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0174f380` | `Wrapper_0174f380 ? → StaticCleanupThunk_01cbda6c` |  | Loads ecx with 01cbda6c and jumps to 0130fd20 (10-byte mov ecx, imm32 / jmp rel32 thunk; the target runs with this=01cbda6c). | [[Native Map/Shards/S0804 0174b4a0-0174f4f0|S0804]] |
| `0174f390` | `Wrapper_0174f390 ? → StaticDtorThunk_01cbdb10` |  | Loads ecx with 01cbdb10 and jumps to 013319c0 (10-byte mov ecx, imm32 / jmp rel32 thunk; the target runs with this=01cbdb10). | [[Native Map/Shards/S0804 0174b4a0-0174f4f0|S0804]] |
| `0174f3a0` | `Wrapper_0174f3a0 ? → StaticDtorThunk_01cbdbb8` |  | Loads ecx with 01cbdbb8 and jumps to 0135ef30 (10-byte mov ecx, imm32 / jmp rel32 thunk; the target runs with this=01cbdbb8). | [[Native Map/Shards/S0804 0174b4a0-0174f4f0|S0804]] |
| `0174f400` | `Wrapper_0174f400 ? → StaticDtorThunk_01cbddb0` |  | Loads ecx with 01cbddb0 and jumps to 013cabd0 (10-byte mov ecx, imm32 / jmp rel32 thunk; the target runs with this=01cbddb0). | [[Native Map/Shards/S0804 0174b4a0-0174f4f0|S0804]] |
| `0174f410` | `Wrapper_0174f410 ? → StaticDtorThunk_01cbddc8` |  | Loads ecx with 01cbddc8 and jumps to 013cabd0 (10-byte mov ecx, imm32 / jmp rel32 thunk; the target runs with this=01cbddc8). | [[Native Map/Shards/S0804 0174b4a0-0174f4f0|S0804]] |
| `0174f440` | `Wrapper_0174f440 ? → StaticDtorThunk_01cbddec` |  | Loads ecx with 01cbddec and jumps to 013cabd0 (10-byte mov ecx, imm32 / jmp rel32 thunk; the target runs with this=01cbddec). | [[Native Map/Shards/S0804 0174b4a0-0174f4f0|S0804]] |
| `0174f450` | `Wrapper_0174f450 ? → StaticDtorThunk_01cbddd4` |  | Loads ecx with 01cbddd4 and jumps to 013cac40 (10-byte mov ecx, imm32 / jmp rel32 thunk; the target runs with this=01cbddd4). | [[Native Map/Shards/S0804 0174b4a0-0174f4f0|S0804]] |
| `0174f460` | `Wrapper_0174f460 ? → StaticCleanupThunk_01cbdda4` |  | Loads ecx with 01cbdda4 and jumps to 013cacb0 (10-byte mov ecx, imm32 / jmp rel32 thunk; the target runs with this=01cbdda4). | [[Native Map/Shards/S0804 0174b4a0-0174f4f0|S0804]] |
| `0174f4b0` | `Wrapper_0174f4b0 ? → StaticDtorThunk_01cbe974` |  | Loads ecx with 01cbe974 and jumps to 014f8790 (10-byte mov ecx, imm32 / jmp rel32 thunk; the target runs with this=01cbe974). | [[Native Map/Shards/S0804 0174b4a0-0174f4f0|S0804]] |

