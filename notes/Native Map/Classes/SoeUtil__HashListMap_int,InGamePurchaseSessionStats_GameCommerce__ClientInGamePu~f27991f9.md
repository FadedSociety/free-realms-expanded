# Class: SoeUtil::HashListMap<int,InGamePurchaseSessionStats<GameCommerce::ClientInGamePurchaseOrder>::IdEntry<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleGroupId,1>_>,16,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b20f20` | `CtorHashListMapIdEntryGroupId16` | ✅ | ctor | Constructor: stores the 16-bucket IdEntry<StoreBundleGroupId> map vtable at +0, zeroes +0x10, +8 and +0xc, memsets 0x40 bytes at +0x14 and zeroes +4; returns the object. |
| `00b24c00` | `— ⚠reject` | ⚪ | virtual | Calls FUN_006fb9c0, then FUN_006fb860. If the thread-local manager is non-null, invokes its vtable slot +4 and returns; otherwise, if DAT_01bdf280 is non-null, invokes that manager slot +4 and returns; otherwise calls F… |
| `00b24c40` | `SoeUtil::HashListMap_StoreBundleGroupIdFreeSlot` | ✅ | virtual | Virtual free slot (vtable index 2): FreeMemory(param_1, 8) on the node pointer. |
| `00b30be0` | `StoreOrderBundleGroupIdMap::ClearNodes` | ✅ | dtor | Writes the map vtable and repeatedly removes its current first node through FUN_00b29300 until the map is empty. |
| `00b310a0` | `StoreOrderBundleGroupIdMap::ScalarDeletingDtor` | ✅ | dtor | Removes nodes through FUN_00b29300, then frees the map allocation when flag bit 0 is set or returns it otherwise. |
| `00b311d0` | `StoreOrderBundleGroupIdMap::ClearNodes` | ✅ | dtor | Installs the specialized map vtable and repeatedly removes its current first node through FUN_00b29300 until empty. |
