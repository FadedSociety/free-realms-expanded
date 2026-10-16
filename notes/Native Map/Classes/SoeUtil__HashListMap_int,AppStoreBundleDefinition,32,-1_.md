# Class: SoeUtil::HashListMap<int,AppStoreBundleDefinition,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b20a60` | `CtorHashListMapAppStoreBundle32` | ✅ | ctor | Constructor: stores SoeUtil::HashListMap<int,AppStoreBundleDefinition,32,-1>::vftable at +0, zeroes +0x10, +8 and +0xc, memsets 0x80 bytes at +0x14 (32 bucket heads) to zero and zeroes +4; returns the object. |
| `00b25560` | `SoeUtil::HashListMap_AppStoreBundleDefinition32AllocSlot ⚠downgrade` | 🟢 | virtual | Calls FUN_006fb9c0, then FUN_006fb860. If the thread-local manager is non-null, invokes its vtable slot +4 and returns; otherwise, if DAT_01bdf280 is non-null, invokes that manager slot +4 and returns; otherwise calls F… |
| `00b255a0` | `SoeUtil::HashListMap_AppStoreBundleDefinition32FreeSlot` | ✅ | virtual | Virtual free slot (vtable index 2): FreeMemory(param_1, 8) on the node pointer. |
| `00b30f90` | `AppStoreBundleDefinitionMap::ClearNodes` | ✅ | dtor | Installs this map vtable and repeatedly removes its current first node through FUN_00b29540 until the map is empty. |
| `00b31360` | `AppStoreBundleDefinitionMap::ScalarDeletingDtor` | ✅ | dtor | Removes nodes through FUN_00b29540, then frees the map allocation when flag bit 0 is set or returns the pointer otherwise. |
| `00b34230` | `AppStoreBundleDefinitionHashMap::Clear` | ✅ | dtor | Installs map vtable, removes the current node repeatedly until the 32-bucket AppStoreBundleDefinition map is empty. |
