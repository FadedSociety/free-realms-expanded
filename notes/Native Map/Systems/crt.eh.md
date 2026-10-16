# System: crt.eh

22 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00427112` | `CxxThrowNullNull → _CxxRethrow_4` |  | MSVC runtime throw wrapper reached from the vector reallocation catch cleanup; the exact exception semantics of the null arguments remain unclear. | [[Native Map/Shards/S0009 00426b13-0042b8f1|S0009]] |
| `00427326` | `CxxThrowNullNull → _CxxRethrow_5` |  | Second MSVC runtime throw wrapper reached from a vector cleanup catch funclet; the exact exception semantics of the null arguments remain unclear. | [[Native Map/Shards/S0009 00426b13-0042b8f1|S0009]] |
| `0042c307` | `CxxRethrow` |  | MSVC rethrow of the exception currently being handled; its only caller is the EH catch funclet Catch@0042c37e. | [[Native Map/Shards/S0010 0042b921-004309b5|S0010]] |
| `0044bc33` | `CxxThrowExceptionNullArgsA` |  | Exception-path helper called from handler 0044bca5; the body proves the null arguments but not a higher-level rethrow meaning. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0044c4d0` | `CxxThrowExceptionNullArgsB` |  | Exception-path helper called from handler 0044c542; the body proves the null arguments but not a higher-level rethrow meaning. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `00457f7e` | `RethrowHelper` |  | Rethrow helper referenced only from the catch funclet at 00457ff5. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |

## 🟡 speculative (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0123aa30` | `Wrapper_0123aa30 ?` |  | Installs a 1-slot EH frame stub, calls FUN_0123ac90 with the stack-cookie XOR value and its argument, then restores ExceptionList; no other work. | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |
| `0123aa80` | `Wrapper_0123aa80 ?` |  | Bytes 55 8b ec 83 ec 64 89 4d 9c 8b 4d 9c e8 ff 01 00 00 8b e5 5d c3: stores ECX to a frame slot, reloads it and calls FUN_0123ac90 with the same this pointer, then returns; a pure forwarding wrapper. | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |
| `0123ae1b` | `SharedEpilogue_0123ae1b ?` |  | Bytes c7 45 fc ff ff ff ff 8b 4d f4 64 89 0d 00 00 00 00 59 5f 5e 5b 8b e5 5d c2 08 00: sets the frame state slot to -1, restores ExceptionList from the frame, pops ecx, edi, esi, ebx, then leave and ret 8; this is a fu… | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |
| `0123b859` | `Tree15_NodeFlagsEpilogue ?` |  | Sets the byte at +0x14 of the node to 1 and the byte at +0x15 to 0, restores ExceptionList and returns the node pointer; a tail of FUN_0123b740 reached through its call site. | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |
| `0123bc20` | `Wrapper_0123bc20 ?` |  | Installs a 1-slot EH frame stub, calls FUN_010935e0 with the stack-cookie XOR value, then restores ExceptionList; no other work. | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |

## ⚪ unknown (11)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00425976` | `— ⚠reject` |  | Compiler-generated/CRT rethrow helper invoked from catch funclets. | [[Native Map/Shards/S0008 00422dcc-00426a69|S0008]] |
| `00425efe` | `— ⚠reject` |  | Compiler-generated/CRT rethrow helper invoked from catch funclets. | [[Native Map/Shards/S0008 00422dcc-00426a69|S0008]] |
| `004268c9` | `— ⚠reject` |  | Compiler-generated/CRT rethrow helper invoked from catch funclets. | [[Native Map/Shards/S0008 00422dcc-00426a69|S0008]] |
| `0047132b` | `— ⚠reject` |  | Compiler-emitted helper for bare throw; inside try/catch statements; reached from the caller at 0047139d. | [[Native Map/Shards/S0023 0046f261-00472f3a|S0023]] |
| `004721de` | `— ⚠reject` |  | Compiler-emitted helper for bare throw; inside try/catch statements; reached from the caller at 00472250. | [[Native Map/Shards/S0023 0046f261-00472f3a|S0023]] |
| `0095b9d0` | `—` |  | Establishes an exception frame, calls FUN_00bf63d0 with the incoming this value still in ECX and no stack arguments, frees param_1 when param_2 bit 0 is set, restores the exception chain, and returns param_1 on the nonf… | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `00a6d910` | `— ⚠reject` |  | Frame helper that the EH unwind funclets call on their cleanup path (16 callers in the map). | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00c34d90` | `— ⚠reject` |  | Invokes 00d605a0 with the stack security-cookie value while unwinding a compiler-generated exception frame. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c35470` | `— ⚠reject` |  | Calls 00d60840 with a stack security-cookie value while unwinding a compiler-generated exception frame. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `00c354c0` | `— ⚠reject` |  | Calls 01086500 with a stack security-cookie value while unwinding a compiler-generated exception frame. | [[Native Map/Shards/S0389 00c34110-00c39a40|S0389]] |
| `01239780` | `— ⚠reject` |  | Installs a 2-slot EH frame stub, calls FUN_01238ae0 with the stack-cookie XOR value as its argument, then restores ExceptionList; it does nothing else. | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |

