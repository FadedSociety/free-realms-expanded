# Class: SoeUtil::HashListMap<int,InGamePurchaseDataSource::MerchantSellBundles::MerchantSellBundleEntry,1024,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00998820` | `MerchantSellBundlesHashListMapCtor` | ✅ | ctor | Fastcall (ECX=this, 47 bytes). Stores the HashListMap<int,InGamePurchaseDataSource::MerchantSellBundles::MerchantSellBundleEntry,1024,-1> vtable (0180d8d0) at [this], zeroes [+8], [+0xc] and [+0x10], calls _memset(this+… |
| `0099bc00` | `HashListMapMerchantSellBundleEntrySlot1Alloc1C` | 🟢 | virtual | Virtual slot [1] of HashListMap<int,InGamePurchaseDataSource::MerchantSellBundles::MerchantSellBundleEntry,1024,-1> (also slot 1 of the HashList vtable at 0180daa0), no stack args, 57 bytes. Calls 006fb9c0, then 006fb86… |
| `0099bc40` | `HashListMapMerchantSellBundleEntrySlot2FreeArg` | 🟢 | virtual | Virtual slot [2] of HashListMap<int,InGamePurchaseDataSource::MerchantSellBundles::MerchantSellBundleEntry,1024,-1>, one stack arg, 18 bytes. Calls 00401c00(arg,4) and returns void. |
| `009a02d0` | `HashListMapMerchantSellDtor` | ✅ | dtor | Installs the typed HashListMap vtable and repeatedly removes head entries with FUN_0099C9A0 until empty. |
| `009a0400` | `HashListMapMerchantSellScalarDtor` | ✅ | virtual | Removes each merchant-sell entry with FUN_0099C9A0, then conditionally frees the map object for the deleting flag. |
| `009a0980` | `HashListMapMerchantSellClear` | ✅ | dtor | Installs the typed merchant-sell map vtable and repeatedly removes the current head entry with FUN_0099C9A0. |
