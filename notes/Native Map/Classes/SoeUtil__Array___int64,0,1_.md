# Class: SoeUtil::Array<__int64,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d63bd0` | `ArrayInt64::Dtor ?` | 🟡 | dtor | Installs the zero-capacity int64 array vtable, resets count, frees backing storage with alignment 8, and clears the pointer. |
| `00d63fa0` | `ArrayInt64::ScalarDeletingDtor ?` | 🟡 | dtor | Resets count, frees int64 storage with alignment eight, clears the pointer, and conditionally frees the array object. |
| `00d641d0` | `ArrayInt64::CopyConstruct ?` | 🟡 | function | Installs the int64 array vtable, initializes storage fields, allocates for the source count, copies the source bytes, and sets the count. |
