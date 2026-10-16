# Class: NewItemsDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c31a20` | `NewItemsDataSourceCtor` | ✅ | ctor | Registers BaseClient.NewItemsData through the common datasource constructor, then installs four NewItemsDataSource vtables. |
| `00c31aa0` | `NewItemsDataSourceDtor` | ✅ | dtor | Restores NewItemsDataSource vtables in four subobjects and calls the shared datasource destructor. |
| `00c31b00` | `NewItemsDataSourceHasRows` | ✅ | method | Checks client/global state and item data, then returns true only when this object's enabled flag is set, the row range is nonempty, and the global eligibility condition is false. |
| `00c31b80` | `NewItemsDataSourceGetFieldName` | ✅ | method | Returns IsRental for field 0x1b, RentalSecondsRemaining for 0x1c, and delegates all other fields to FUN_00cb09d0. |
| `00c31bb0` | `NewItemsDataSourceDtorAdjustor` | ✅ | method | Subtracts 0x254 from ECX and jumps to 00c31bc0, adjusting this for an inherited-base destructor entry. |
| `00c31bc0` | `NewItemsDataSourceScalarDeletingDtor` | ✅ | dtor | Calls the derived datasource destructor, then frees this when flag bit 0 is set. |
| `00c31be0` | `NewItemsDataSourceGetFieldValue` | ✅ | method | For field 0x1b, resolves the row object and formats whether its item instance differs from the empty sentinel; for 0x1c, formats remaining rental seconds or zero; other fields delegate to FUN_00cb1160. |
