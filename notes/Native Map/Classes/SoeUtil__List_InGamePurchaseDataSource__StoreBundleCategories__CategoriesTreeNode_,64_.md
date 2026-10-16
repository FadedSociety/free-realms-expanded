# Class: SoeUtil::List<InGamePurchaseDataSource::StoreBundleCategories::CategoriesTreeNode*,64>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009a2640` | `CategoriesTreeNodeListCtor` | ✅ | ctor | Installs the category-node list vtables, zeros its head, tail, and count fields, and initializes the list tracking state. |
| `009a2ca0` | `CategoriesTreeNodeListDestructor` | ✅ | dtor | Installs the bounded list vtable, drains list entries, clears list fields, recursively frees both root child subtrees, frees the root, and restores the unbounded base-list vtable. |
| `009a2d70` | `CategoryNodeListPopAdjustor` | ✅ | virtual | Adds 0x10 to this and jumps to 009a2bd0, the free-list pop implementation. |
| `009a2d80` | `CategoryNodeListPushAdjustor` | ✅ | virtual | Adds 0x10 to this and jumps to 009a2c40, the free-list push implementation. |
| `009a2e10` | `CategoryNodeListScalarDeletingDtor` | ✅ | dtor | Calls the category-node list destructor, tests deletion flag bit 0, frees this when set, and otherwise returns this. |
