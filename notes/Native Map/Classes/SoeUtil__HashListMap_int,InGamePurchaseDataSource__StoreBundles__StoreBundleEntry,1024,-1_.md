# Class: SoeUtil::HashListMap<int,InGamePurchaseDataSource::StoreBundles::StoreBundleEntry,1024,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00998730` | `StoreBundlesHashListMapCtor` | ✅ | ctor | Fastcall (ECX=this, 47 bytes). Stores the HashListMap<int,InGamePurchaseDataSource::StoreBundles::StoreBundleEntry,1024,-1> vtable (0180d8b0) at [this], zeroes [+8], [+0xc] and [+0x10], calls _memset(this+0x14, 0, 0x100… |
| `0099bb30` | `HashListMapStoreBundleEntrySlot1Alloc24` | 🟢 | virtual | Virtual slot [1] of HashListMap<int,InGamePurchaseDataSource::StoreBundles::StoreBundleEntry,1024,-1> (also the slot 1 of the HashList<StoreBundleEntry,1024,-1> vtable), no stack args, 57 bytes. Calls 006fb9c0, then 006… |
| `0099bb70` | `HashListMapStoreBundleEntrySlot2FreeArg` | 🟢 | virtual | Virtual slot [2] of HashListMap<int,InGamePurchaseDataSource::StoreBundles::StoreBundleEntry,1024,-1>, one stack arg, 18 bytes. Calls 00401c00(arg,4) and returns void. |
| `009a02a0` | `HashListMapStoreBundlesDtor` | ✅ | dtor | Installs the typed HashListMap vtable and repeatedly removes the head entry with FUN_0099C720 until the map is empty. |
| `009a03c0` | `HashListMapStoreBundlesScalarDtor` | ✅ | virtual | Removes all entries with FUN_0099C720, then conditionally frees the map object when the scalar-deleting flag is set. |
| `009a0ba0` | `HashListMapStoreBundlesClear` | ✅ | dtor | Installs the typed StoreBundles map vtable and repeatedly removes the current head entry with FUN_0099C720. |
