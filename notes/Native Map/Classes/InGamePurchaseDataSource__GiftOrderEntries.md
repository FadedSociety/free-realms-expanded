# Class: InGamePurchaseDataSource::GiftOrderEntries

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00997e00` | `GiftOrderEntriesFieldNameForIndex` | ✅ | virtual | Virtual slot [1] of GiftOrderEntries, one stack arg (index), no calls. Index 0 returns StoreId, 1 StoreBundleId, 2 Quantity; any other index returns "<error>". |
| `0099d670` | `GiftOrderEntries::InitThunk` | ✅ | virtual | Adjusts ECX by -0x254 and jumps directly to 0099ACF0; this is an adjustor thunk, not the target body. |
| `009a43a0` | `GiftOrderEntries::GetValue` | ✅ | virtual | Resolves the requested gift-order entry, checks its index against the record count, and writes one of the supported entry fields to the output. |
| `009a71c0` | `InGamePurchaseDataSource::GiftOrderEntries::~GiftOrderEntries` | ✅ | dtor | Destructor body: writes GiftOrderEntries::vftable to this[0], [0xc], [0xd], [0x95], then overwrites this[0x95] with InternalDsBase::vftable and calls FUN_0091bf60; performs no delete. |
| `009a7d20` | `InGamePurchaseDataSource::GiftOrderEntries::GiftOrderEntries` | ✅ | ctor | Constructor: calls the base constructor FUN_00922890(param_2), stores InternalDsBase::vftable at [0x95], the owner word param_3 at [0x96], installs GiftOrderEntries::vftable at [0], [0xc], [0xd], [0x95] and zeroes [0x97… |
