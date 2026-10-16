# Class: CollectionCategoryManager::CollectionCategoryDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009fce30` | `CollectionCategoryDataSource::ColumnName` | ✅ | virtual | Returns one of four fixed headers based on the column index: Collection Category ID, NAME, ICON_ID, or the generic Collection Category label. |
| `009fd0b0` | `CollectionCategoryDataSource::GetValue` | ✅ | virtual | Returns the dword at offset +0xc of the object referenced by this+0x224. |
| `009fd7a0` | `CollectionCategoryDataSource::SetValue` | ✅ | virtual | Gets a selected row, looks up its category node, then writes the selected category ID, localized name, or icon ID for columns 0, 1, or 2. |
| `009fd8d0` | `CollectionCategoryManager::CollectionCategoryDataSource::Dtor` | ✅ | dtor | Restores the data-source vtable at the primary and two embedded offsets, then destroys the embedded user-data table. |
| `009fd930` | `CollectionCategoryManager::CollectionCategoryDataSource::Ctor` | ✅ | ctor | Registers BaseClient.CollectionCategory, initializes the primary and two embedded data-source vtables, and stores its manager pointer at +0x254. |
| `009fd9b0` | `CollectionCategoryManager::CollectionCategoryDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the collection category data-source destructor, frees the object when flag bit 0 is set, and otherwise returns its pointer. |
