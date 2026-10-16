# Class: InGamePurchaseDataSource::SubscriptionPriceList

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00997f20` | `SubscriptionPriceListFieldNameForIndex` | ✅ | virtual | Virtual slot [1] of SubscriptionPriceList, one stack arg (index), no calls. Switch: 0 Id, 1 Name, 2 Description, 3 PriceLocalCurrency, 4 PriceStationCash, 5 IsVatTaxable, 6 Sku, 7 MarketingDescription, 8 ProductLegalTex… |
| `0099ad80` | `SubscriptionPriceListSlot1RefreshFrom3b980` | 🟢 | virtual | Virtual slot [1] of SubscriptionPriceList, fastcall ECX=this, no stack args, 23 bytes. Reads [[this+4]+0x1c8]; if nonzero, copies the dword at that object+0x3b980 into this+8. Returns void. |
| `009a4680` | `— ⚠reject` | ⚪ | virtual | Bounds-checks the selected subscription row and handles selectors 0 through 10 by returning stored strings, numbers, flags, or formatted price fields. |
| `009a7400` | `InGamePurchaseDataSource::SubscriptionPriceList::~SubscriptionPriceList` | ✅ | dtor | Destructor body: writes SubscriptionPriceList::vftable to this[0], [0xc], [0xd], [0x95], then overwrites this[0x95] with InternalDsBase::vftable and calls FUN_0091bf60; performs no delete. |
| `009a7f60` | `InGamePurchaseDataSource::SubscriptionPriceList::SubscriptionPriceList` | ✅ | ctor | Constructor: calls the base constructor FUN_00922890(param_2), stores InternalDsBase::vftable at [0x95], the owner word param_3 at [0x96], installs SubscriptionPriceList::vftable at [0], [0xc], [0xd], [0x95] and zeroes … |
