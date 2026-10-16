# Class: SoeUtil::HashMap<int,int,16384,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0098b300` | `HashMap16384Int::Destructor` | ✅ | dtor | Installs the 16384-bucket map vtable, clears integer entries, frees its tree root and child links, restores the base map vtable, and clears entries again. |
| `0098b5f0` | `HashMap16384Int::ScalarDeletingDestructor` | ✅ | dtor | Calls the 16384-bucket integer-map destructor body; frees this only when deleting flag bit 0 is set, otherwise returns this. |
| `0098b6b0` | `HashMap16384Int::Constructor` | ✅ | ctor | Installs the base map vtable, zeros the 16384 bucket array and counters, then installs the pooled-map vtable and clears its pool state. |
