# Class: SoeUtil::HashMap<int,InGamePurchaseSessionStats<GameCommerce::ClientInGamePurchaseOrder>::OrderTracker*,8,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b25a40` | `SoeUtil::HashMapOrderTrackerDtor` | ✅ | dtor | 11-byte tail-call body (called from four EH unwind funclets): mov [ecx],0181e1b0 then jmp FUN_00b21e90 - i.e. it writes SoeUtil::HashMap<int,...,OrderTracker*,8,-1>::vftable to *this and tail-jumps to the map body relea… |
| `00b25aa0` | `SoeUtil::HashMapOrderTrackerAllocSlot` | ✅ | virtual | Calls FUN_006fb9c0, then FUN_006fb860. If the thread-local manager is non-null, invokes its vtable slot +4 and returns; otherwise, if DAT_01bdf280 is non-null, invokes that manager slot +4 and returns; otherwise calls F… |
| `00b25ae0` | `SoeUtil::HashMapOrderTrackerFreeSlot` | ✅ | virtual | Virtual free slot (vtable index 2): FreeMemory(param_1, 4) on the node pointer. |
| `00b25d60` | `SoeUtil::HashMapOrderTrackerScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): writes the HashMap vtable to *this, releases the map body via FUN_00b21e90, then frees this via _free when the flags low bit is set. |
| `00b28ae0` | `SoeUtil::HashMapOrderTrackerDtorVariant` | ✅ | dtor | SEH-framed destructor variant (called from an unwind funclet): writes the HashMap vtable, saves this into the frame and releases the map body through FUN_00b21e90, restoring ExceptionList. |
| `00b28b80` | `SoeUtil::HashMapOrderTrackerCtor` | ✅ | ctor | Constructor: writes the HashMap vtable, zeroes the ten header/state dwords at +4..+0x28, then writes the base SoeUtil::Hash<OrderTracker*,8,-1>::vftable. |
| `00b28c60` | `SoeUtil::HashMapOrderTrackerScalarDeletingDtorVariant` | ✅ | dtor | SEH-framed scalar deleting destructor variant (holds slot 0 of the base Hash<OrderTracker*,8,-1> vtable): writes the HashMap vtable, releases the map body via FUN_00b21e90 and frees this via _free when the flags low bit… |
| `00b31670` | `OrderTrackerHashMap::ProcessEntries` | ✅ | method | Walks linked entries, compares each key pair with global values, updates matching fields, checks related tracker state, removes selected hash-chain nodes, and calls the key-processing helper. |
