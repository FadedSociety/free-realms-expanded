# Class: InGamePurchaseDataSource::StoreBillboardPanels

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00997cc0` | `StoreBillboardPanelsFieldNameForIndex` | ✅ | virtual | Virtual slot [1] of StoreBillboardPanels, one stack arg (index), no calls. Switch: 0 Id, 1 BillboardId, 2 Name, 3 ImageId, 4 IsMembersOnly, 5 ShortcutType, 6 ShortcutValue, 7 IsVislble (spelling as in the binary); any o… |
| `0099f4b0` | `StoreBillboardPanels::AdjustorThunk` | ✅ | virtual | Adjusts ECX by -0x254 then jumps directly to 0099D560; no panel processing body exists in this thunk. |
| `009a3f30` | `StoreBillboardPanels::GetValue` | ✅ | virtual | Resolves the selected panel row, indexes a panel record, and handles its supported field selectors, including a converted value for selector 7. |
| `009a6e70` | `InGamePurchaseDataSource::StoreBillboardPanels::~StoreBillboardPanels` | ✅ | dtor | Destructor body: writes StoreBillboardPanels::vftable to this[0], this[0xc], this[0xd], this[0x95], then overwrites this[0x95] with InternalDsBase::vftable and calls the base-object teardown FUN_0091bf60 under the stack… |
| `009a7a10` | `InGamePurchaseDataSource::StoreBillboardPanels::StoreBillboardPanels` | ✅ | ctor | Constructor: calls the base constructor FUN_00922890(param_2), stores InternalDsBase::vftable at [0x95], the owner word param_3 at [0x96], installs StoreBillboardPanels::vftable at [0], [0xc], [0xd], [0x95] and initiali… |
