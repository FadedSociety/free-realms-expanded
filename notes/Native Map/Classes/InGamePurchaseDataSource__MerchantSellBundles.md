# Class: InGamePurchaseDataSource::MerchantSellBundles

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009980f0` | `MerchantSellBundlesFieldNameForIndex` | ✅ | virtual | Virtual slot [1] of MerchantSellBundles, one stack arg (index). Index 0x22 returns QuantityAvailable; any other index calls 00997840 and returns its result. 26 bytes in the Ghidra body. |
| `009a6490` | `MerchantSellBundles::GetValue` | ✅ | virtual | Requires the backing client data, looks up the row key in a 1024-bucket chain for selector 0x22, writes the found value or zero, and delegates other selectors to the shared store-value method. |
| `009a8cd0` | `InGamePurchaseDataSource::MerchantSellBundles::MerchantSellBundles` | ✅ | ctor | Constructor: calls the StoreBundles base constructor FUN_009a8880(param_2, param_3, 1), installs MerchantSellBundles::vftable at [0], [0xc], [0xd], [0x95], [0x9c], constructs the MerchantSellBundleEntry map via FUN_009a… |
| `009a8dc0` | `InGamePurchaseDataSource::MerchantSellBundles::~MerchantSellBundles` | ✅ | dtor | MerchantSellBundles destructor: writes MerchantSellBundles::vftable to [0], [0xc], [0xd], [0x95], [0x9c], destroys the MerchantSellBundleEntry hash map via FUN_009a0980 under EH state 0, then calls the StoreBundles base… |
