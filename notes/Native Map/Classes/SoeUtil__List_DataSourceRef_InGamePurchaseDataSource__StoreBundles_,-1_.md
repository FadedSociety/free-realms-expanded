# Class: SoeUtil::List<DataSourceRef<InGamePurchaseDataSource::StoreBundles>,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0099ba50` | `ListDataSourceRefStoreBundlesSlot2Alloc12` | 🟢 | virtual | Virtual slot [2] of SoeUtil::List<DataSourceRef<InGamePurchaseDataSource::StoreBundles>,-1>, no stack args, 57 bytes. Calls 006fb9c0, then 006fb860: if that is nonzero calls its slot +4 and returns; else if DAT_01bdf280… |
| `0099ba90` | `ListDataSourceRefStoreBundlesSlot3FreeArg` | 🟢 | virtual | Virtual slot [3] of SoeUtil::List<DataSourceRef<InGamePurchaseDataSource::StoreBundles>,-1>, one stack arg, 18 bytes. Calls 00401c00(arg,4) and returns void. |
| `009a4b20` | `StoreBundlesDataSourceRefListDestructor` | ✅ | dtor | Writes the typed reference-list vtable at this, then tail-jumps to 009a32e0 to unlink entries, release references, and clean nodes. |
| `009a4cd0` | `StoreBundlesDataSourceRefListScalarDeletingDtor` | ✅ | dtor | Installs the typed list vtable, clears and releases its entries, tests deletion flag bit 0, frees this when set, and otherwise returns this. |
