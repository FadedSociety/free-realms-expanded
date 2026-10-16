# Class: SoeUtil::HashListMap<int,ActionBarSlot,8,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c43e50` | `— ⚠reject` | ⚪ | virtual | Virtual allocator slot of the 8-bucket action-bar map: obtains a memory manager (FUN_006fb9c0 then FUN_006fb860) and either allocates 0x68 bytes with alignment 8 through FUN_006fb950 (its own manager), or defers to the … |
| `00c43e90` | `HashListMap8m1_FreeNode` | 🟢 | virtual | Virtual free slot of the same map: builds the two stack arguments (pointer, 8) and calls the FreeMemory helper FUN_00401c00, returning with ret 4. |
| `00c44530` | `HashListMap8m1_Destruct` | 🟢 | dtor | Destructor body: reinstalls the HashListMap<int,ActionBarSlot,8,-1> vftable and repeatedly frees the node at +8 through FUN_008cae00 (the loop body re-reads +8, so the helper is what advances the list). |
| `00c44560` | `HashListMap8m1_ScalarDeletingDtor` | 🟢 | virtual | Virtual slot 0 deleting destructor: does the same vtable reinstall and node freeing as 00c44530 (through FUN_008cae00) and then, when the low bit of the flag argument is set, frees the map object itself with _free. |
