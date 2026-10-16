# Class: SoeUtil::HashMap<int,int,8,6>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0098a360` | `— ⚠reject` | ⚪ | ctor | Initializes the eight-bucket HashMap base and derived vtables, zeros the map state, chains eight inline pool records, and stores pool capacity 6. |
| `0098b4a0` | `HashMap8InlinePool::Destructor` | ✅ | dtor | Installs the HashMap vtable, clears its eight integer buckets, frees its tree root and child links, restores the base vtable, and clears the buckets again. |
| `0098b630` | `HashMap8InlinePool::ScalarDeletingDestructor` | ✅ | dtor | Calls the eight-bucket integer-map destructor body; frees this only when deleting flag bit 0 is set, otherwise returns this. |
