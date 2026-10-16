# Class: SoeUtil::Array<SoeUtil::StringFixed<64>,4,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00beffe0` | `SoeUtil::Array::FreeStorage` | ✅ | virtual | Compares the supplied pointer with the object's aligned inline buffer and frees it with alignment 4 only when it is external. |
| `00bf0010` | `SoeUtil::ArrayStringFixed64::Dtor` | ✅ | dtor | Destroys array elements, releases external storage, switches to the zero-inline-array base vtable, then destroys and frees its storage. |
| `00bf00b0` | `SoeUtil::ArrayStringFixed64::Allocate` | ✅ | virtual | For capacity at most four returns the aligned inline buffer and sets alignment 4; larger capacities delegate allocation to FUN_0095ae70. |
| `00bf02b0` | `SoeUtil::ArrayStringFixed64::Ctor` | ✅ | ctor | Installs the zero-inline array base, clears its three storage and size fields, then installs the four-inline-element array vtable. |
| `00bf0310` | `SoeUtil::ArrayStringFixed64::ScalarDeletingDtor` | ✅ | dtor | Calls the array destructor, frees the object when flag bit 0 is set, and otherwise returns its pointer. |
| `00bf06c0` | `SoeUtil::ArrayStringFixed64::Assign` | ✅ | virtual | Copies element storage from the source and destination arrays through FUN_0096b240, then frees their old storage and clears temporary array state. |
