# Class: SoeUtil::HashMap<int,SoeUtil::Hash<int,8,6>,4096,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0098cce0` | `HashMap4096NestedPooled::Destructor` | ✅ | dtor | Installs the pooled-map vtable, clears nested entries, frees its tree root and children, restores the base map vtable, and clears the buckets again. |
| `0098d760` | `HashMap4096NestedPooled::ScalarDeletingDestructor` | ✅ | dtor | Calls the pooled nested-map destructor; frees this only when deleting flag bit 0 is set, otherwise returns this. |
| `0098d780` | `HashMap4096NestedPooled::Constructor` | ✅ | ctor | Installs the base-map vtable, zeroes the 4096 bucket array and counters, then installs the pooled nested-map vtable and clears pool state. |
