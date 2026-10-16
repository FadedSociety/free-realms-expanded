# Class: InGamePurchaseDataSource::StoreBundleGroupContents

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009a3a50` | `StoreBundleGroupContents::Refresh` | ✅ | virtual | Removes current rows, marks the data source dirty, resolves the backing group record, and appends each contained row key with the current source value. |
| `009a84c0` | `InGamePurchaseDataSource::StoreBundleGroupContents::~StoreBundleGroupContents` | ✅ | dtor | Destructor: writes StoreBundleGroupContents::vftable to [0], [0xc], [0xd], [0x95], [0x9c], then calls the StoreBundles base destructor FUN_009a83b0 under the stack-cookie check; no delete. |
| `009a89d0` | `InGamePurchaseDataSource::StoreBundleGroupContents::StoreBundleGroupContents` | ✅ | ctor | Constructor: calls the StoreBundles base constructor FUN_009a8880(param_2, param_3, 1), then installs StoreBundleGroupContents::vftable at [0], [0xc], [0xd], [0x95], [0x9c], sets [0x4b7] = DAT_01be590c and clears byte +… |
