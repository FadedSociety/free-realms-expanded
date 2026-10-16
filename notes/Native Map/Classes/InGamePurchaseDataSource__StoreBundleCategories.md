# Class: InGamePurchaseDataSource::StoreBundleCategories

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009a37b0` | `StoreBundleCategoriesCtor` | ✅ | ctor | Constructs the internal data-source base, installs the StoreBundleCategories vtable in both object subobjects, stores its supplied member, and initializes its free-pool state. |
| `009a3840` | `StoreBundleCategoriesDeletingDtorAdjustor` | ✅ | dtor | Subtracts 0x38 from this and jumps to 009a6c10, the StoreBundleCategories scalar deleting destructor. |
| `009a6710` | `StoreBundleCategoriesDestructor` | ✅ | dtor | Restores the category data-source vtables, recycles the owned node, destroys the embedded category-node list and tree root, restores the base data-source vtable, and releases its base object. |
| `009a67c0` | `StoreBundleCategories::RefreshTree` | ✅ | virtual | Recycles the prior tree, checks backing category data and count, creates root and category nodes, indexes category records, recursively builds descendants, and clears temporary heap storage. |
| `009a6c10` | `StoreBundleCategoriesScalarDeletingDtor` | ✅ | dtor | Calls the StoreBundleCategories destructor, tests deletion flag bit 0, frees this when set, and otherwise returns this. |
