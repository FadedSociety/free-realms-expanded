# Class: InGamePurchaseDataSource::GiftOrderHistory

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00997da0` | `GiftOrderHistoryFieldNameForIndex` | ✅ | virtual | Virtual slot [1] of GiftOrderHistory, one stack arg (index), no calls. Switch: 0 Id, 1 PurchaserGuid, 2 PurchaserName, 3 GiftMessage, 4 TimeStamp, 5 ItemEntryCount; any other index returns "<error>". |
| `0099f4c0` | `GiftOrderHistory::AdjustorThunk` | ✅ | virtual | Adjusts ECX by -0x254 and jumps to 0099D600, the gift-history row traversal routine. |
| `009a41f0` | `GiftOrderHistory::GetValue` | ✅ | virtual | Resolves the selected order-history record, dispatches selectors 0 through 5, copies stored values, formats the elapsed-time field, and returns whether the record was available. |
| `009a7100` | `InGamePurchaseDataSource::GiftOrderHistory::~GiftOrderHistory` | ✅ | dtor | Destructor body: writes GiftOrderHistory::vftable to this[0], [0xc], [0xd], [0x95], then overwrites this[0x95] with InternalDsBase::vftable and calls FUN_0091bf60 (base-object teardown) under the stack-cookie check; per… |
| `009a7c60` | `InGamePurchaseDataSource::GiftOrderHistory::GiftOrderHistory` | ✅ | ctor | Constructor: calls the base constructor FUN_00922890(param_2), stores InternalDsBase::vftable at [0x95], the owner word param_3 at [0x96], installs GiftOrderHistory::vftable at [0], [0xc], [0xd], [0x95] and zeroes [0x97… |
