# Class: InGamePurchaseDataSource::CountryCodes

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00998050` | `CountryCodesFieldNameForIndex` | ✅ | virtual | Virtual slot [1] of CountryCodes, one stack arg (index), no calls. Switch: 0 Id, 1 CountryCode, 2 LanguageLocale, 3 CountryLocale, 4 CountryName; any other index returns "<error>". |
| `0099adc0` | `CountryCodesSlot1RefreshFrom3ce08` | 🟢 | virtual | Virtual slot [1] of CountryCodes, fastcall ECX=this, no stack args, 23 bytes. Reads [[this+4]+0x1c8]; if nonzero, copies the dword at that object+0x3ce08 into this+8. Returns void. |
| `009a4990` | `— ⚠reject` | ⚪ | virtual | Bounds-checks the selected country row and writes one of its stored fields for selectors 0 through 4. |
| `009a7580` | `InGamePurchaseDataSource::CountryCodes::~CountryCodes` | ✅ | dtor | Destructor body: writes CountryCodes::vftable to this[0], [0xc], [0xd], [0x95], then overwrites this[0x95] with InternalDsBase::vftable and calls FUN_0091bf60; performs no delete. |
| `009a80e0` | `InGamePurchaseDataSource::CountryCodes::CountryCodes` | ✅ | ctor | Constructor: calls the base constructor FUN_00922890(param_2), stores InternalDsBase::vftable at [0x95], the owner word param_3 at [0x96], installs CountryCodes::vftable at [0], [0xc], [0xd], [0x95] and zeroes [0x97]; r… |
