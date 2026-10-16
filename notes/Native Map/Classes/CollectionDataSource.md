# Class: CollectionDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c31f00` | `CollectionDataSourceFieldCount` | ✅ | method | Calls FUN_00992cd0, adds nine, and returns the maximum field index used by the datasource. |
| `00c31f10` | `CollectionDataSourceGetFieldName` | ✅ | method | Maps field indices 0-8 to collection ID, name, description, image ID, two unused columns, entry count, image tint and category ID; later valid indices return Entry Collected. |
| `00c326c0` | `CollectionDataSourceGetFieldValue` | ✅ | method | Refreshes rows if flagged, gets the requested collection row, then dispatches by field index to obtain localized name/description, image, counts, tint, category or collected-state values. |
| `00c328b0` | `— ⚠reject` | ⚪ | method | When the refresh flag is set, repopulates the collection rows, then tail-jumps to FUN_010c8f30 with ECX adjusted to this+4. |
| `00c32dd0` | `CollectionDataSourceDtor` | ✅ | dtor | Restores CollectionDataSource vtables in its main and adjusted subobjects, then tears down the embedded GUI datasource base at +0x34. |
| `00c32ea0` | `CollectionDataSourceCtor` | ✅ | ctor | Registers BaseClient.CollectionData, installs the class vtables, stores two supplied datasource pointers, sets an enabled flag, and clears a state field. |
| `00c32f30` | `CollectionDataSourceScalarDeletingDtor` | ✅ | dtor | Calls CollectionDataSource cleanup and frees the object when flag bit 0 is set. |
