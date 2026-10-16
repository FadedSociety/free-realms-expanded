# Class: InGamePurchaseDataSource::StateCodes

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009980b0` | `StateCodesFieldNameForIndex` | ✅ | virtual | Virtual slot [1] of StateCodes, one stack arg (index), no calls. Index 0 returns Id, 1 StateCode, 2 StateName; any other index returns "<error>". |
| `0099ade0` | `StateCodesSlot1RefreshFrom3d61c` | 🟢 | virtual | Virtual slot [1] of StateCodes, fastcall ECX=this, no stack args, 23 bytes. Reads [[this+4]+0x1c8]; if nonzero, copies the dword at that object+0x3d61c into this+8. Returns void. |
| `009a4a80` | `— ⚠reject` | ⚪ | virtual | Bounds-checks the selected state row and writes its first or second stored field for selectors 0 through 2. |
| `009a7640` | `InGamePurchaseDataSource::StateCodes::~StateCodes` | ✅ | dtor | Destructor body: writes StateCodes::vftable to this[0], [0xc], [0xd], [0x95], then overwrites this[0x95] with InternalDsBase::vftable and calls FUN_0091bf60; performs no delete. |
| `009a81a0` | `InGamePurchaseDataSource::StateCodes::StateCodes` | ✅ | ctor | Constructor: calls the base constructor FUN_00922890(param_2), stores InternalDsBase::vftable at [0x95], the owner word param_3 at [0x96], installs StateCodes::vftable at [0], [0xc], [0xd], [0x95] and zeroes [0x97]; ret… |
