# Class: InGamePurchaseDataSource::StoreBundleCategories::CategoriesTreeNode

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00997ab0` | `CategoriesTreeNodeSlot4FieldName` | ✅ | virtual | Virtual slot [4] of CategoriesTreeNode, one stack arg (undefined4 index), no calls. Returns a static name: 0 CategoryId, 1 StoreBundleCount, 2 TintValue; any other index returns "UNKNOWN". |
| `0099ffa0` | `CategoriesTreeNodeDtor` | ✅ | dtor | Restores the category-node vtable, runs a member cleanup helper, changes the embedded field to Loggable's vtable, and invokes the GUI tree-node cleanup body. |
| `009a0020` | `CategoriesTreeNodeGetMember34` | ✅ | virtual | Returns the address this+0x34. |
| `009a0050` | `CategoriesTreeNodeGetField28` | ✅ | virtual | Returns the 32-bit value stored at this+0x28. |
| `009a0d90` | `CategoriesTreeNodeCtor` | ✅ | ctor | Calls GUI tree-node base constructor 0099EF20 with this in ECX, installs the category-node vtable, stores supplied identifiers and fields, constructs icon data, and resolves a string with a numeric fallback. |
| `009a0e50` | `CategoriesTreeNodeScalarDtor` | ✅ | virtual | Calls the category-node cleanup body and conditionally frees the object when the deleting flag is set. |
| `009a38a0` | `StoreBundleCategories::GetCategoryValue` | ✅ | virtual | For field selectors 0, 1, and 2, writes node fields or a converted value to the output; unsupported selectors write -1 and return zero. |
