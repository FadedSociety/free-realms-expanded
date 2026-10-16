# Class: InGamePurchaseDataSource::StationCashPriceList

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00997eb0` | `StationCashPriceListFieldNameForIndex` | ✅ | virtual | Virtual slot [1] of StationCashPriceList, one stack arg (index), no calls. Switch: 0 Id, 1 TotalStationCashAmount, 2 PromotionalStationCashAmount, 3 PromotionalStationCashGranted, 4 RetailPrice, 5 SalePrice, 6 IsOnSale;… |
| `0099ad60` | `StationCashPriceListSlot1RefreshFrom3b5d8` | 🟢 | virtual | Virtual slot [1] of StationCashPriceList, fastcall ECX=this, no stack args, 23 bytes. Reads [[this+4]+0x1c8]; if nonzero, copies the dword at that object+0x3b5d8 into this+8. Returns void. |
| `009a4540` | `— ⚠reject` | ⚪ | virtual | Bounds-checks the selected price row, writes row values for selectors 0 through 6, tests whether sale price is positive, and formats selected price fields. |
| `009a7340` | `InGamePurchaseDataSource::StationCashPriceList::~StationCashPriceList` | ✅ | dtor | Destructor body: writes StationCashPriceList::vftable to this[0], [0xc], [0xd], [0x95], then overwrites this[0x95] with InternalDsBase::vftable and calls FUN_0091bf60; performs no delete. |
| `009a7ea0` | `InGamePurchaseDataSource::StationCashPriceList::StationCashPriceList` | ✅ | ctor | Constructor: calls the base constructor FUN_00922890(param_2), stores InternalDsBase::vftable at [0x95], the owner word param_3 at [0x96], installs StationCashPriceList::vftable at [0], [0xc], [0xd], [0x95] and zeroes [… |
