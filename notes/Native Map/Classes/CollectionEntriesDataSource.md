# Class: CollectionEntriesDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c31fd0` | `CollectionEntriesDataSourceGetFieldName` | ✅ | method | Returns one of seven field names: collection ID, entry ID, name, image-set ID, index, collected state, or tint value; otherwise returns <error>. |
| `00c322f0` | `CollectionEntriesDataSourceGetCount` | ✅ | method | If refresh flag +0x22c is set, calls the row-population routine; returns field +0xc from the object at +0x228 or zero. |
| `00c328d0` | `CollectionEntriesDataSourceGetFieldValue` | ✅ | method | Refreshes rows when flagged, resolves the requested entry row, and returns its name/localization, image set, index, collected state, tint, or other configured fields by index. |
| `00c32e30` | `CollectionEntriesDataSourceDtor` | ✅ | dtor | Restores CollectionEntriesDataSource vtables in its subobjects, then tears down the embedded GUI datasource base. |
| `00c32f50` | `CollectionEntriesDataSourceCtor` | ✅ | ctor | Registers BaseClient.CollectionEntriesData, installs class vtables, stores the configuration pointer, clears state, and sets an enabled flag. |
| `00c32fe0` | `CollectionEntriesDataSourceScalarDeletingDtor` | ✅ | dtor | Calls CollectionEntriesDataSource teardown and frees the object when flag bit 0 is set. |
