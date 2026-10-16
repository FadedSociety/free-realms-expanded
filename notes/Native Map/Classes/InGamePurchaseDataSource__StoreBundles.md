# Class: InGamePurchaseDataSource::StoreBundles

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00997840` | `StoreBundleFieldNameForIndex` | ✅ | virtual | Virtual, one stack arg (undefined4 index), ret 4, no calls. Switches on the index and returns a pointer to a static field-name string: 0 ItemId, 1 StoreId, 2 CategoryId, 3 Name, 4 Description, 5 Price, 6 ImageSetId, 7 I… |
| `009a3650` | `StoreBundlesRefreshDerivedEntries` | ✅ | virtual | When its refresh flag is set, clears the existing entry map, finds the matching store bundle record, appends each derived record key with the current source value, and marks the data dirty. |
| `009a56d0` | `StoreBundles::GetValue` | ✅ | virtual | Handles selectors 0 through 0x21, returning stored bundle fields, formatted strings, flags, availability predicates, and values derived from associated item records. |
| `009a6650` | `StoreBundles::ResolveAndRequestBundleRecord` | ✅ | virtual | Rebuilds filtered rows when dirty, resolves the requested bundle record through its selected object, requests missing item data when the record needs it, and returns the selected lookup result. |
| `009a66f0` | `StoreBundles::GetCount` | ✅ | virtual | Rebuilds filtered rows when dirty, then returns the value stored at this plus 0x24C. |
| `009a83b0` | `InGamePurchaseDataSource::StoreBundles::~StoreBundles` | ✅ | dtor | StoreBundles destructor: writes StoreBundles::vftable to [0], [0xc], [0xd], [0x95], [0x9c]; under EH states destroys the embedded SearchFilter HashListMap (FUN_009a8340), the StoreBundleEntry HashListMap (FUN_009a0ba0) … |
| `009a8880` | `InGamePurchaseDataSource::StoreBundles::StoreBundles` | ✅ | ctor | StoreBundles constructor: base ctor FUN_00922890(param_2), member init FUN_0081b470, InternalDsBase::vftable at [0x9c] and owner word param_3 at [0x9d], then StoreBundles::vftable at [0], [0xc], [0xd], [0x95], [0x9c]; s… |
