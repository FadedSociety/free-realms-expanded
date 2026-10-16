# Class: SoeUtil::List<SoccerPlayerStatus,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b75c30` | `ListSoccerPlayerStatus::AllocNode` | ✅ | virtual | Vtable slot 2 of the list's allocator interface: takes the current thread's memory manager (FUN_006fb9c0 then FUN_006fb860) and allocates through its vtable slot +4, falls back to the global manager object DAT_01bdf280 … |
| `00b75c70` | `ListSoccerPlayerStatus::FreeNode` | ✅ | virtual | Vtable slot 3 of the list's allocator interface: frees a node by calling FUN_00401c00(node, 8) — the FreeMemory route with alignment 8 (18-byte body, no other work). |
| `00b7dce0` | `SoccerPlayerStatusListCleanup` | ✅ | dtor | Repeatedly reads the list field at +4 and calls 00b7ac20 on it until it becomes zero; it does not free the outer list object. |
| `00b7dd80` | `SoccerPlayerStatusListScalarDeletingDtor` | ✅ | dtor | Installs the typed list vtable, repeatedly destroys the nonzero field at +4, returns this when flag bit 0 is clear, and calls free on this when flag bit 0 is set. |
| `00b7fb50` | `SoccerPlayerStatusListDtor → SoccerPlayerStatusListInline8Dtor` | ✅ | dtor | Destroys SoccerPlayerStatus entries, recursively frees both node-pool tree branches and their blocks, clears pool fields, then installs the generic list vtable and drains remaining list nodes. |
