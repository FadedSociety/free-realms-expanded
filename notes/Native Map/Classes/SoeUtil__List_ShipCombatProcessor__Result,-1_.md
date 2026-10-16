# Class: SoeUtil::List<ShipCombatProcessor::Result,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a53cc0` | `SoeUtil::List_Result_AllocateNode` | ✅ | virtual | Virtual node-allocation hook (slot 2) of List<ShipCombatProcessor::Result,-1>: calls FUN_006fb9c0, then uses the current thread's memory manager from FUN_006fb860 (its virtual slot 1) when non-null, else the global mana… |
| `00a53d00` | `SoeUtil::List_Result_FreeNode` | ✅ | virtual | Virtual node-free hook (slot 3) of List<ShipCombatProcessor::Result,-1>: forwards the node pointer to FUN_00401c00 with alignment 4. |
| `00a58350` | `SoeUtil::List_Result_Dtor` | ✅ | dtor | Writes the typed Result-list vtable, then machine code tail-jumps to FUN_00a56880; bytes c7 01 9c 4b 81 01 e9 25 e5 ff ff decode as mov [ecx],0x01814b9c; jmp 00a56880. Ghidra pseudocode incorrectly renders call-return h… |
| `00a58360` | `SoeUtil::List_Result_ScalarDeletingDtor` | ✅ | dtor | Installs the Result-list vtable, clears its elements, and frees the containing list object when flag bit 0 is set. |
