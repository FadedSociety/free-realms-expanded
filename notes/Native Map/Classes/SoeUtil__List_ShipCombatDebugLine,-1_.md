# Class: SoeUtil::List<ShipCombatDebugLine,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a53750` | `SoeUtil::List_DebugLine_AllocateNode` | ✅ | virtual | Virtual node-allocation hook (slot 2) of List<ShipCombatDebugLine,-1>: calls FUN_006fb9c0, then uses the current thread's memory manager from FUN_006fb860 (virtual slot 1) when non-null, else the global manager DAT_01bd… |
| `00a53790` | `SoeUtil::List_DebugLine_FreeNode` | ✅ | virtual | Virtual node-free hook (slot 3) of List<ShipCombatDebugLine,-1>: forwards the node pointer to FUN_00401c00 with alignment 0x10. |
| `00a59bf0` | `ShipCombatDebugLineListDtor` | ✅ | dtor | Stores the debug-line list vtable pointer, then tail-jumps to 00a58180 for cleanup. |
| `00a59da0` | `ShipCombatDebugLineListScalarDtor` | ✅ | dtor | Installs the list vtable, runs list cleanup, and frees the list object when flag bit 0 is set. |
