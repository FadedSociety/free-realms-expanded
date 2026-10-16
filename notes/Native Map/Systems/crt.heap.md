# System: crt.heap

7 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00481081` | `AllocateBlock56` |  | Fixed-size heap-allocation wrapper. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `0048108a` | `FreePointerArgument` |  | CRT free wrapper; the body does not identify the freed block's record type. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |

## 🟡 speculative (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ad3360` | `ScalarDeletingDtor ?` |  | Scalar deleting destructor wrapper; allocator operation follows object cleanup. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `013c2a30` | `FreeMemberAt8 ?` |  | Calls _free on the pointer stored at this+8 (bytes 8b 41 08 / 50 / e8 ... ff: mov eax,[ecx+8]; push eax; call _free). The 9-byte window has no ret, so the exact return path is not shown here. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |

## ⚪ unknown (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b4ab30` | `— ⚠reject` |  | Compiler-generated cleanup step for an object that owns a heap buffer at +0xc, extracted as its own function and called from four unwind funclets in the screenshot code. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b4acb0` | `— ⚠reject` |  | SEH-flavoured twin of 00b4ab30, used as a cleanup body by the screenshot buffer path (caller 00b4b3e0). | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `013c2230` | `— ⚠reject` |  | Frees the pointer stored in the first word of the object at param_1 with _free when that word is nonzero; otherwise does nothing. It is called from 73 EH unwind funclets. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |

