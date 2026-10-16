# Class: InGamePurchaseDataSource::StoreBundleActiveFilters

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00997d50` | `StoreBundleActiveFiltersFieldNameForIndex` | ✅ | virtual | Virtual slot [1] of StoreBundleActiveFilters, one stack arg (index), no calls. Index 0 returns FilterId, 1 returns OptionList; any other index returns "<error>". |
| `009a40a0` | `— ⚠reject` | ⚪ | virtual | Calls the base row-reset virtual, zeros its count field, resolves the store bundle object, visits its filter array, and increments the count for each visited item. |
| `009a4110` | `StoreBundleActiveFilters::GetValue` | ✅ | virtual | Resolves the requested filter entry and backing bundle, then returns a selected value or writes the ordered filter identifiers into the output. |
| `009a6f30` | `InGamePurchaseDataSource::StoreBundleActiveFilters::~StoreBundleActiveFilters` | ✅ | dtor | Destructor body: writes StoreBundleActiveFilters::vftable to this[0], [0xc], [0xd], [0x95], then under EH state 1 calls FUN_00402140 (SoeUtil::IStringFixed256 dtor) on the object, resets this[0x95] to InternalDsBase::vf… |
| `009a7ad0` | `InGamePurchaseDataSource::StoreBundleActiveFilters::StoreBundleActiveFilters ⚠d…` | 🟢 | ctor | Constructor: base ctor FUN_00922890(param_2), InternalDsBase::vftable at [0x95], owner word param_3 at [0x96], StoreBundleActiveFilters::vftable at [0], [0xc], [0xd], [0x95], then assigns the source-name SoeUtil::String… |
