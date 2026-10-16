# Class: SoeUtil::List<ShipCombatPlayer,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a52f70` | `SoeUtil::List_ShipCombatPlayer_Allocate` | ✅ | virtual | Virtual Allocate hook (slot 2) of List<ShipCombatPlayer,-1>: uses the thread/global block allocator when present, else allocates a node through FUN_006fb950 with element size 0x360 and alignment 8. |
| `00a52fb0` | `SoeUtil::List_ShipCombatPlayer_Deallocate` | ✅ | virtual | Virtual Deallocate hook (slot 3) of List<ShipCombatPlayer,-1>: forwards the node pointer to FreeMemory with alignment 8 (bytes 8b 44 24 04 6a 08 50 match the two-argument free wrapper). |
| `00a6afc0` | `List_ShipCombatPlayer_Destruct` | ✅ | dtor | Destructor for the list object: stores the SoeUtil::List<ShipCombatPlayer,-1> vtable, then repeatedly calls 00a68330 on the head at +4 until the list is empty. |
| `00a6b010` | `List_ShipCombatPlayer_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor for the ShipCombatPlayer list: rewrites the list vtable, drains the head through 00a68330, then conditionally frees this when delete-flag bit 0 is set. |
