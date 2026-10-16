# Class: RentalsDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c30cf0` | `RentalsDataSourceCtor` | ✅ | ctor | Calls the common datasource base constructor, installs RentalsDataSource vtables at four subobject offsets, and stores the supplied configuration pointer at +0x2dc. |
| `00c30d90` | `RentalsDataSourceDtor` | ✅ | dtor | Restores the RentalsDataSource vtable in four subobjects and calls the common datasource destructor. |
| `00c30df0` | `RentalsDataSourceGetFieldName` | ✅ | method | For field 0x1b returns MarketplaceBundleId; all other fields delegate to FUN_00cb09d0. |
| `00c30f30` | `RentalsDataSourceDtorAdjustor` | ✅ | method | Subtracts 0x254 from ECX and jumps to 00c30f40, adjusting this before the shared destructor. |
| `00c30f40` | `RentalsDataSourceScalarDeletingDtor` | ✅ | dtor | Calls the RentalsDataSource destructor and frees the object when flag bit 0 is set. |
| `00c30f60` | `RentalsDataSourceIsUnavailable ? ⚠rename` | 🟡 | method | If field +0x28 is non-null and differs from two global values, calls FUN_0103dfe0; returns 1 only when that helper returns false, otherwise returns 0. |
| `00c30fc0` | `RentalsDataSourceGetValue` | ✅ | method | For field 0x1b returns item+0x28 child field +0xc or zero through the output formatter; other fields delegate to FUN_00cb1500. |
