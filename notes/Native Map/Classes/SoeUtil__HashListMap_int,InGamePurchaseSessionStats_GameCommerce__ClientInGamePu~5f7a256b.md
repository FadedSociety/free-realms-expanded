# Class: SoeUtil::HashListMap<int,InGamePurchaseSessionStats<GameCommerce::ClientInGamePurchaseOrder>::IdEntry<int>,16,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b21110` | `CtorHashListMapIdEntryInt16` | ✅ | ctor | Constructor: stores the 16-bucket IdEntry<int> map vtable at +0, zeroes +0x10, +8 and +0xc, memsets 0x40 bytes at +0x14 and zeroes +4; returns the object. |
| `00b24cc0` | `— ⚠reject` | ⚪ | virtual | Calls FUN_006fb9c0, then FUN_006fb860. If the thread-local manager is non-null, invokes its vtable slot +4 and returns; otherwise, if DAT_01bdf280 is non-null, invokes that manager slot +4 and returns; otherwise calls F… |
| `00b24d00` | `SoeUtil::HashListMap_IdEntryInt16FreeSlot` | ✅ | virtual | Virtual free slot (vtable index 2): FreeMemory(param_1, 8) on the node pointer. |
| `00b30c40` | `StoreOrderIntMap16::ClearNodes` | ✅ | dtor | Writes this map vtable and repeatedly removes the current node through FUN_00b29460 until no nodes remain. |
| `00b31120` | `PurchaseOrderIntMap16::ScalarDeletingDtor` | ✅ | dtor | Removes nodes through FUN_00b29460, then frees the map allocation when flag bit 0 is set or returns it otherwise. |
| `00b312b0` | `PurchaseOrderIntMap16::ClearNodes` | ✅ | dtor | Installs the specialized map vtable and repeatedly removes its current first node through FUN_00b29460 until empty. |
