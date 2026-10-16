# Class: InGamePurchaseDataSource::StoreBundleActiveSort

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00997d80` | `StoreBundleActiveSortFieldNameForIndex` | ✅ | virtual | Virtual slot [1] of StoreBundleActiveSort, one stack arg (index), no calls. Index 0 returns SortId; any other index returns "<error>". Bytes-equivalent to the Ghidra body (20 bytes). |
| `009a41a0` | `StoreBundleActiveSort::GetValue` | ✅ | virtual | Resolves the StoreBundles data source, writes its selected sort field for selector zero, and returns success when the source exists. |
| `009a7010` | `InGamePurchaseDataSource::StoreBundleActiveSort::~StoreBundleActiveSort` | ✅ | dtor | Destructor body: writes StoreBundleActiveSort::vftable to this[0], [0xc], [0xd], [0x95], then under EH state 1 calls FUN_00402140 (IStringFixed256 dtor), resets this[0x95] to InternalDsBase::vftable and calls FUN_0091bf… |
| `009a7ba0` | `InGamePurchaseDataSource::StoreBundleActiveSort::StoreBundleActiveSort` | ✅ | ctor | Constructor: base ctor FUN_00922890(param_2), InternalDsBase::vftable at [0x95], owner word param_3 at [0x96], StoreBundleActiveSort::vftable at [0], [0xc], [0xd], [0x95], then assigns the source-name StringFixed<256> v… |
