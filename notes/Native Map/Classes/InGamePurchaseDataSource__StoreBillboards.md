# Class: InGamePurchaseDataSource::StoreBillboards

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00997c80` | `StoreBillboardsFieldNameForIndex` | ✅ | virtual | Virtual slot [1] of StoreBillboards, one stack arg (index), no calls. Index 0 returns ColumnId, 1 LocationId, 2 PanelCount; any other index returns "<error>". |
| `0099f4a0` | `StoreBillboards::AdjustorThunk` | ✅ | virtual | Adjusts ECX by -0x254 and jumps directly to 0099D510, which performs the linked-list traversal. |
| `009a3e80` | `StoreBillboards::GetValue` | ✅ | virtual | Checks the backing data object, resolves the selected billboard entry, and handles the supported field selectors before returning success or failure. |
| `009a6db0` | `InGamePurchaseDataSource::StoreBillboards::~StoreBillboards` | ✅ | dtor | Destructor body: writes StoreBillboards::vftable to this[0], this[0xc], this[0xd], this[0x95], then overwrites this[0x95] with InternalDsBase::vftable and calls the base-object teardown FUN_0091bf60 under the stack-cook… |
| `009a7960` | `InGamePurchaseDataSource::StoreBillboards::StoreBillboards` | ✅ | ctor | Constructor: calls the base constructor FUN_00922890(param_2), stores InternalDsBase::vftable at [0x95], the owner word param_3 at [0x96], then installs StoreBillboards::vftable at [0], [0xc], [0xd], [0x95]; returns thi… |
