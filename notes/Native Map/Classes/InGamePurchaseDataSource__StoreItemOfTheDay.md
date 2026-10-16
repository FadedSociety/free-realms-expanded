# Class: InGamePurchaseDataSource::StoreItemOfTheDay

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00997b60` | `StoreItemOfTheDayFieldNameForIndex` | ✅ | virtual | Virtual slot [1] of StoreItemOfTheDay, one stack arg (index). Switch: 0x22 QuantityAvailable, 0x23 TimeRemaining, 0x24 IOTDPrice, 0x25 Message. Any other index calls 00997840 (the store bundle field-name lookup) and ret… |
| `009a62b0` | `StoreItemOfTheDay::GetValue` | ✅ | virtual | Requires the backing client data, returns three stored fields for selectors 0x22 through 0x24, checks selector 0x25, and delegates all other selectors to the shared store-value method. |
| `009a85e0` | `InGamePurchaseDataSource::StoreItemOfTheDay::~StoreItemOfTheDay` | ✅ | dtor | Destructor: writes StoreItemOfTheDay::vftable to [0], [0xc], [0xd], [0x95], [0x9c], then rewrites the member at this+0x4b8 (byte 0x12e0) from GameCommerce::StoreItemOfTheDay<AppStoreBundleDefinition>::vftable to GameCom… |
| `009a8b50` | `InGamePurchaseDataSource::StoreItemOfTheDay::StoreItemOfTheDay` | ✅ | ctor | Constructor: calls the StoreBundles base constructor FUN_009a8880(param_2, param_3, 1), installs StoreItemOfTheDay::vftable at [0], [0xc], [0xd], [0x95], [0x9c], then constructs the embedded definition member at +0x4b8 … |
