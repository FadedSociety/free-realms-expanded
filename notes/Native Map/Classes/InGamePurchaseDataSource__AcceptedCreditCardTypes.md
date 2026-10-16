# Class: InGamePurchaseDataSource::AcceptedCreditCardTypes

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00997fc0` | `AcceptedCreditCardTypesFieldNameForIndex` | ✅ | virtual | Virtual slot [1] of AcceptedCreditCardTypes, one stack arg (index), no calls. Switch: 0 Id, 1 CardName, 2 CardSymbol, 3 CardIconId, 4 CsvHelperIconId, 5 CardNumberLength, 6 CsvLength, 7 CsvRequired, 8 CardIssuerRequired… |
| `0099ada0` | `AcceptedCreditCardTypesSlot1RefreshFrom3ca34` | 🟢 | virtual | Virtual slot [1] of AcceptedCreditCardTypes, fastcall ECX=this, no stack args, 23 bytes. Reads [[this+4]+0x1c8]; if nonzero, copies the dword at that object+0x3ca34 into this+8. Returns void. |
| `009a4850` | `— ⚠reject` | ⚪ | virtual | Bounds-checks the selected credit-card type row and writes its fields for selectors 0 through 8. |
| `009a74c0` | `InGamePurchaseDataSource::AcceptedCreditCardTypes::~AcceptedCreditCardTypes` | ✅ | dtor | Destructor body: writes AcceptedCreditCardTypes::vftable to this[0], [0xc], [0xd], [0x95], then overwrites this[0x95] with InternalDsBase::vftable and calls FUN_0091bf60; performs no delete. |
| `009a8020` | `InGamePurchaseDataSource::AcceptedCreditCardTypes::AcceptedCreditCardTypes` | ✅ | ctor | Constructor: calls the base constructor FUN_00922890(param_2), stores InternalDsBase::vftable at [0x95], the owner word param_3 at [0x96], installs AcceptedCreditCardTypes::vftable at [0], [0xc], [0xd], [0x95] and zeroe… |
