# Class: SoeUtil::HashListMap<int,InGamePurchaseSessionStats<GameCommerce::ClientInGamePurchaseOrder>::IdEntry<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleId,1>_>,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b20e20` | `CtorHashListMapIdEntryBundleId32` | ✅ | ctor | Constructor: stores the 32-bucket IdEntry<StoreBundleId> map vtable at +0, zeroes +0x10, +8 and +0xc, memsets 0x80 bytes at +0x14 and zeroes +4; returns the object. |
| `00b24ba0` | `— ⚠reject` | ⚪ | virtual | Calls FUN_006fb9c0, then FUN_006fb860. If the thread-local manager is non-null, invokes its vtable slot +4 and returns; otherwise, if DAT_01bdf280 is non-null, invokes that manager slot +4 and returns; otherwise calls F… |
| `00b24be0` | `SoeUtil::HashListMap_StoreBundleIdFreeSlot` | ✅ | virtual | Virtual free slot (vtable index 2): FreeMemory(param_1, 8) on the node pointer. |
| `00b30bb0` | `StoreOrderBundleIdMap::ClearNodes` | ✅ | dtor | Writes the map vtable and repeatedly removes the current first node through FUN_00b29240 until the map is empty. |
| `00b31060` | `StoreOrderBundleIdMap::ScalarDeletingDtor` | ✅ | dtor | Removes all nodes through FUN_00b29240, then frees the map allocation when flag bit 0 is set or returns it otherwise. |
| `00b31160` | `StoreOrderBundleIdMap::ClearNodes` | ✅ | dtor | Installs the specialized map vtable and repeatedly removes its current first node through FUN_00b29240 until empty. |
