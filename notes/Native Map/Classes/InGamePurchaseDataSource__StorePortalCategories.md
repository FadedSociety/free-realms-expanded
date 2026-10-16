# Class: InGamePurchaseDataSource::StorePortalCategories

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00997c00` | `StorePortalCategoriesFieldNameForIndex` | ✅ | virtual | Virtual slot [1] of StorePortalCategories, one stack arg (index), no calls. Switch: 0 Id, 1 Name, 2 Description, 3 ImageSetId, 4 TintValue, 5 ShortcutType, 6 ShortcutValue, 7 ShortcutStoreId; any other index returns "<e… |
| `0099f490` | `StorePortalCategories::AdjustorThunk` | ✅ | virtual | Adjusts ECX by -0x254 and jumps directly to 0099D4C0, a shared row-population body. |
| `009a3cd0` | `StorePortalCategories::GetValue` | ✅ | virtual | Looks up the selected portal-category record, dispatches field selectors 0 through 7, writes the selected value or string conversion, and returns whether the record was found. |
| `009a6cf0` | `InGamePurchaseDataSource::StorePortalCategories::~StorePortalCategories` | ✅ | dtor | Destructor body: writes StorePortalCategories::vftable to this[0], this[0xc], this[0xd] and this[0x95] (byte offsets 0, 0x30, 0x34, 0x254), then overwrites this[0x95] with InternalDsBase::vftable and calls the base-obje… |
| `009a78b0` | `InGamePurchaseDataSource::StorePortalCategories::StorePortalCategories` | ✅ | ctor | Constructor: calls the base constructor FUN_00922890(param_2), stores InternalDsBase::vftable at [0x95], the owner word param_3 at [0x96], then installs StorePortalCategories::vftable at [0], [0xc], [0xd], [0x95]; retur… |
