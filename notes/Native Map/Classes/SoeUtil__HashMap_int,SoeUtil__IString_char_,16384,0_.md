# Class: SoeUtil::HashMap<int,SoeUtil::IString<char>,16384,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0098be80` | `HashMap16384StringPooled::Destructor` | ✅ | dtor | Installs the pooled-map vtable, clears string-valued buckets, frees its tree root and children, restores the base vtable, and clears the buckets again. |
| `0098c0e0` | `HashMap16384StringPooled::ScalarDeletingDestructor` | ✅ | dtor | Calls the pooled 16384-bucket string-map destructor; frees this only when deleting flag bit 0 is set, otherwise returns this. |
| `0098c120` | `HashMap16384StringPooled::Constructor` | ✅ | ctor | Installs the base map vtable, zeros the 16384-bucket array and counters, then installs the pooled-map vtable and clears its pool state. |
