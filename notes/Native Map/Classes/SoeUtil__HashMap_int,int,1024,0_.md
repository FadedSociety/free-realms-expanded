# Class: SoeUtil::HashMap<int,int,1024,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0098b3d0` | `HashMap1024Int::Destructor` | ✅ | dtor | Installs the 1024-bucket map vtable, clears integer entries and the tree root, restores the base map vtable, and clears the entries again. |
| `0098b610` | `HashMap1024Int::ScalarDeletingDestructor` | ✅ | dtor | Calls the 1024-bucket integer-map destructor body; frees this only when deleting flag bit 0 is set, otherwise returns this. |
| `0098b740` | `HashMap1024Int::Constructor` | ✅ | ctor | Installs the base map vtable, zeros the 1024 bucket array and counters, then installs the pooled-map vtable and clears its pool state. |
