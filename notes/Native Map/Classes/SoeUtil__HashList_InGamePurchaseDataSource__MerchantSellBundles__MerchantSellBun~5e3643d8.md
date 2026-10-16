# Class: SoeUtil::HashList<InGamePurchaseDataSource::MerchantSellBundles::MerchantSellBundleEntry,1024,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009a1ad0` | `HashListMerchantSellCtor` | ✅ | ctor | Initializes the typed HashListMap, zeroes map fields and 0x1000 bucket bytes, then installs the HashList vtable. |
| `009a2dd0` | `MerchantSellHashListScalarDeletingDtor` | ✅ | dtor | Clears the MerchantSellBundles hash list, tests deletion flag bit 0, frees this when set, and otherwise returns this. |
