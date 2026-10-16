# Class: SoeUtil::Array<double,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d63d00` | `ArrayDouble::Dtor ?` | 🟡 | dtor | Installs the zero-capacity double-array vtable, resets count, frees backing storage with alignment 8, and clears the pointer. |
| `00d64040` | `ArrayDouble::ScalarDeletingDtor ?` | 🟡 | dtor | Resets count, frees double storage with alignment eight, clears the pointer, and conditionally frees the array object. |
| `00d642e0` | `ArrayDouble::CopyConstruct ?` | 🟡 | function | Installs the double array vtable, initializes storage fields, allocates for the source count, copies source bytes, and sets the count. |
