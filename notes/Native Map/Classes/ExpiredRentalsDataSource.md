# Class: ExpiredRentalsDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c30e10` | `ExpiredRentalsDataSourceCtor` | ✅ | ctor | Calls the RentalsDataSource constructor, then installs ExpiredRentalsDataSource vtables in four subobjects. |
| `00c30ea0` | `ExpiredRentalsDataSourceDtor` | ✅ | dtor | Restores ExpiredRentalsDataSource vtables at four subobject offsets, then calls RentalsDataSource teardown. |
| `00c30f00` | `ExpiredRentalsDataSourceCanRefresh ? ⚠rename` | 🟡 | method | Calls FUN_0103dfe0 and returns a normalized boolean indicating whether it returned nonzero. |
| `00c30f20` | `ExpiredRentalsDataSourceDtorAdjustor` | ✅ | method | Subtracts 0x254 from ECX, then jumps to 00c30fa0; it is an adjustor thunk, not a separate destructor body. |
| `00c30fa0` | `ExpiredRentalsDataSourceScalarDeletingDtor` | ✅ | dtor | Runs expired-rentals teardown, then frees the object when flag bit 0 is set. |
