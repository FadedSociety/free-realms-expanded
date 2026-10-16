# Class: SoeUtil::List<FlashEntity*,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d55d60` | `ListPoolCallSlot1 ?` | 🟡 | virtual | Calls FUN_006fb9c0, then FUN_006fb860 to obtain a pool object; if non-null calls its vtable slot 1, else if the global DAT_01bdf280 is non-null calls its vtable slot 1, else calls FUN_006fb950(0xc,4). |
| `00d55da0` | `ListFreeBlock4 ?` | 🟡 | virtual | Calls FreeMemory (FUN_00401c00) with the argument and alignment 4, then returns with ret 4. |
| `00d56640` | `ListDtor → SoeUtil::List<FlashEntity*,-1>::Dtor` | ✅ | dtor | Sets the List<FlashEntity*> vtable in *param_1 and clears its node chain through ListClearNodes (00d562d0); also called from two EH unwind funclets. |
| `00d566f0` | `ListScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of the FlashEntity list: resets the vtable, calls ListClearNodes (00d562d0), then calls _free(this) when bit 0 of the flags argument is set. |
