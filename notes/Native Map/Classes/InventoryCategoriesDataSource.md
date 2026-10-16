# Class: InventoryCategoriesDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a6dd00` | `— ⚠reject` | ⚪ | dtor | Destructor body: stores InventoryCategoriesDataSource::vftable at this; when the pointer at +0x30 is non-null it releases that object through 010cdbd0 and its virtual slot 0 with argument 1; then calls 00822810 with the… |
| `00a6de90` | `InventoryCategoriesDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls the destructor body 00a6dd00, then frees this through _free when bit 0 of the flag argument is set; returns this. |
| `00a6e480` | `InventoryCategoriesDataSource::RebuildCategoryTree` | 🟢 | method | Releases the object at +0x30 (010cdbd0, then its slot 0 with argument 1), creates the root CategoriesTreeNode through 00a6e170 with arguments 0, 0, 0, 1 when the allocation 00d833a6 succeeds, links it with 010cd820, the… |
| `00a6e9d0` | `InventoryCategoriesDataSource::InventoryCategoriesDataSource` | ✅ | ctor | Constructor: calls 009974a0 with the BaseClient.InventoryCategoriesData name, stores param_2 at +0x38 (the keyed category table), installs the InventoryCategoriesDataSource vtable at offset 0, then calls 00a6e480 to bui… |
