# Class: SoeUtil::HashListMap<int,IgnoreData,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cf8380` | `HashListMapIgnoreDataInit ?` | 🟡 | init | Installs the map vtable, clears map metadata, zeroes 64 bucket pointers, and returns the initialized object. |
| `00cf8520` | `HashListMapIgnoreDataAllocateNode ?` | 🟡 | virtual | Initializes allocator state, requests a node from the current memory manager or global fallback, and otherwise calls the common heap allocator. |
| `00cf8df0` | `HashListMapIgnoreDataScalarDeletingDtor ?` | 🟡 | dtor | Restores the map vtable, removes all linked IgnoreData nodes through FUN_00cf89d0, and frees the object when the delete flag is set. |
