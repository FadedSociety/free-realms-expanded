# Class: SoeUtil::HashMap<int,InGamePurchaseSessionStats<GameCommerce::ClientInGamePurchaseOrder>::TransactionTracker*,8,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b25b00` | `SoeUtil::HashMapTransactionTrackerDtor` | ✅ | dtor | 11-byte tail-call body (called from four EH unwind funclets): mov [ecx],0181e1c0 then jmp FUN_00b21f20 - it writes the TransactionTracker* HashMap vtable and tail-jumps to the map body release. |
| `00b25b60` | `SoeUtil::HashMapTransactionTrackerAllocSlot` | ✅ | virtual | Calls FUN_006fb9c0, then FUN_006fb860. If the thread-local manager is non-null, invokes its vtable slot +4 and returns; otherwise, if DAT_01bdf280 is non-null, invokes that manager slot +4 and returns; otherwise calls F… |
| `00b25ba0` | `SoeUtil::HashMapTransactionTrackerFreeSlot` | ✅ | virtual | Virtual free slot (vtable index 2): FreeMemory(param_1, 4) on the node pointer. |
| `00b25d90` | `SoeUtil::HashMapTransactionTrackerScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): writes the HashMap vtable to *this, releases the map body via FUN_00b21f20, then frees this via _free when the flags low bit is set. |
| `00b28b30` | `SoeUtil::HashMapTransactionTrackerDtorVariant` | ✅ | dtor | SEH-framed destructor variant: writes the HashMap vtable, saves this in the frame and releases the map body through FUN_00b21f20. |
| `00b28bf0` | `SoeUtil::HashMapTransactionTrackerCtor` | ✅ | ctor | Constructor: writes the HashMap vtable, zeroes the ten header/state dwords, then writes the base SoeUtil::Hash<TransactionTracker*,8,-1>::vftable. |
| `00b28cc0` | `SoeUtil::HashMapTransactionTrackerScalarDeletingDtorVariant` | ✅ | dtor | SEH-framed scalar deleting destructor variant (holds slot 0 of the base Hash<TransactionTracker*,8,-1> vtable): writes the HashMap vtable, releases the map body via FUN_00b21f20 and conditionally frees this via _free. |
