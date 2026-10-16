# Class: InGamePurchaseDataSource::StoreBundleOfTheDay

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00997bb0` | `StoreBundleOfTheDayFieldNameForIndex` | ✅ | virtual | Virtual slot [1] of StoreBundleOfTheDay, one stack arg (index). Switch: 0x22 QuantityAvailable, 0x23 TimeRemaining, 0x24 IOTDPrice, 0x25 Message. Any other index calls 00997840 and returns its result. Same body as 00997… |
| `009a63a0` | `StoreBundleOfTheDay::GetValue` | ✅ | virtual | Requires the backing client data, returns three stored fields for selectors 0x22 through 0x24, checks selector 0x25, and delegates other selectors to the shared store-value method. |
| `009a86e0` | `InGamePurchaseDataSource::StoreBundleOfTheDay::~StoreBundleOfTheDay` | ✅ | dtor | Destructor: writes StoreBundleOfTheDay::vftable to [0], [0xc], [0xd], [0x95], [0x9c], then rewrites the member at this+0x4b8 (byte 0x12e0) from GameCommerce::StoreBundleOfTheDay<AppStoreBundleDefinition>::vftable to Gam… |
| `009a8c10` | `InGamePurchaseDataSource::StoreBundleOfTheDay::StoreBundleOfTheDay` | ✅ | ctor | Constructor: calls the StoreBundles base constructor FUN_009a8880(param_2, param_3, 1), installs StoreBundleOfTheDay::vftable at [0], [0xc], [0xd], [0x95], [0x9c], then constructs the embedded definition member at +0x4b… |
