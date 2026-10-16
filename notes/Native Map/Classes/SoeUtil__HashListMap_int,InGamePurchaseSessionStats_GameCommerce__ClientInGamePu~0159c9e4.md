# Class: SoeUtil::HashListMap<int,InGamePurchaseSessionStats<GameCommerce::ClientInGamePurchaseOrder>::IdEntry<int>,6,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b24c60` | `— ⚠reject` | ⚪ | virtual | Calls FUN_006fb9c0, then FUN_006fb860. If the thread-local manager is non-null, invokes its vtable slot +4 and returns; otherwise, if DAT_01bdf280 is non-null, invokes that manager slot +4 and returns; otherwise calls F… |
| `00b24ca0` | `SoeUtil::HashListMap_IdEntryInt6FreeSlot` | ✅ | virtual | Virtual free slot (vtable index 2): FreeMemory(param_1, 8) on the node pointer. |
| `00b30c10` | `StoreOrderIntMap6::ClearNodes ? ⚠rename` | 🟡 | dtor | Writes this map vtable and repeatedly removes the current node through FUN_00b293c0 until no nodes remain. |
| `00b310e0` | `PurchaseOrderIntMap6::ScalarDeletingDtor` | ✅ | dtor | Removes nodes through FUN_00b293c0, then frees the map allocation when flag bit 0 is set or returns it otherwise. |
| `00b31240` | `PurchaseOrderIntMap6::ClearNodes` | ✅ | dtor | Installs the specialized map vtable and repeatedly removes its current first node through FUN_00b293c0 until empty. |
