# Class: SoeUtil::List<ShipCombatDebugSphere,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a536e0` | `— ⚠reject` | ⚪ | virtual | Virtual node-allocation hook (slot 2) of List<ShipCombatDebugSphere,-1>: calls FUN_006fb9c0, then asks FUN_006fb860 for the current thread's memory manager and, when non-null, calls its virtual slot 1 (the allocate meth… |
| `00a53720` | `SoeUtil::List_DebugSphere_FreeNode` | ✅ | virtual | Virtual node-free hook (slot 3) of List<ShipCombatDebugSphere,-1>: loads the node pointer argument and forwards it to FUN_00401c00 with alignment 0x10 (bytes 8b 44 24 04 6a 10 50 e8 ... c2 04 00). |
| `00a59bb0` | `ShipCombatDebugSphereListDtor` | ✅ | dtor | Stores the debug-sphere list vtable pointer, then tail-jumps to 00a58120 for cleanup. |
| `00a59d70` | `ShipCombatDebugSphereListScalarDtor` | ✅ | dtor | Installs the list vtable, runs list cleanup, and frees the list object when flag bit 0 is set. |
