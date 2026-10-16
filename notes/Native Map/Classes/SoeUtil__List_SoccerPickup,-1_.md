# Class: SoeUtil::List<SoccerPickup,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b75d30` | `ListSoccerPickup::AllocNode` | ✅ | virtual | Vtable slot 2 of the pickup list's allocator interface: allocates through the current thread memory manager's vtable slot +4 (via FUN_006fb9c0/FUN_006fb860), falls back to the global manager DAT_01bdf280, then to FUN_00… |
| `00b75d70` | `ListSoccerPickup::FreeNode` | ✅ | virtual | Vtable slot 3 of the pickup list's allocator interface: frees a node by calling FUN_00401c00(node, 4) — FreeMemory with alignment 4 (18-byte body, no other work). |
