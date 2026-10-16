# Class: SoeUtil::HashMap<int,SoeUtil::IString<char>,1024,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0098bf50` | `HashMap1024StringPooled::Destructor` | ✅ | dtor | Installs the pooled-map vtable, clears string-valued buckets, frees its tree root and children, restores the base vtable, and clears the buckets again. |
| `0098c100` | `HashMap1024StringPooled::ScalarDeletingDestructor` | ✅ | dtor | Calls the pooled 1024-bucket string-map destructor; frees this only when deleting flag bit 0 is set, otherwise returns this. |
| `0098c1b0` | `HashMap1024StringPooled::Constructor` | ✅ | ctor | Installs the base map vtable, zeros the 1024-bucket array and counters, then installs the pooled-map vtable and clears its pool state. |
