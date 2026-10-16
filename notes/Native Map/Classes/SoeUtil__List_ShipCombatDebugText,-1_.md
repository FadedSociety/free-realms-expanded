# Class: SoeUtil::List<ShipCombatDebugText,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a537c0` | `SoeUtil::List_DebugText_AllocateNode` | ✅ | virtual | Virtual node-allocation hook (slot 2) of List<ShipCombatDebugText,-1>: calls FUN_006fb9c0, then the thread memory manager from FUN_006fb860 (slot 1) when non-null, else the global manager DAT_01bdf280's slot 1, else the… |
| `00a53800` | `SoeUtil::List_DebugText_FreeNode` | ✅ | virtual | Virtual node-free hook (slot 3) of List<ShipCombatDebugText,-1>: forwards the node pointer to FUN_00401c00 with alignment 0x10. |
| `00a64a90` | `ShipCombatDebugTextList_Dtor` | ✅ | dtor | Destructor body of the ship-combat debug text list: installs SoeUtil::List<ShipCombatDebugText,-1>::vftable at +0 and drains the node list at +4 by repeatedly calling the node remover 00a5d0f0 until the head is null. |
| `00a64cd0` | `ShipCombatDebugTextList_ScalarDtor` | ✅ | dtor | Scalar deleting destructor of the debug text list: installs SoeUtil::List<ShipCombatDebugText,-1>::vftable, drains the +4 node list with 00a5d0f0 and, when bit 0 of param_2 is set, releases the list object through the _… |
