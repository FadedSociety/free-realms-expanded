# Class: SoeUtil::HashList<InGamePurchaseDataSource::StoreBundles::StoreBundleEntry,1024,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009a1a50` | `HashListStoreBundlesCtor` | ✅ | ctor | Initializes the typed HashListMap, zeroes bucket pointers and size fields, clears 0x1000 bytes of bucket array, then installs the HashList vtable. |
| `009a2db0` | `StoreBundlesHashListScalarDeletingDtor` | ✅ | dtor | Clears the StoreBundles hash list, tests deletion flag bit 0, frees this when set, and otherwise returns this. |
