# Class: SoeUtil::HashListMap<int,int,512,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cfb410` | `HashListMapInt512Init ?` | 🟡 | init | Installs the HashListMap vtable, clears metadata and zeroes the 512 bucket pointers. |
| `00cfb7a0` | `HashListMapInt512RemoveNode ?` | 🟡 | function | Unlinks a node from the 512-bucket chain and doubly linked list, decrements the count, then calls the map's virtual node deallocator. |
| `00cfb810` | `HashListMapInt512AllocateNode ?` | 🟡 | virtual | Initializes allocator state, requests a node from the configured allocator or global fallback, then calls the heap allocator if neither exists. |
| `00cfb850` | `HashListMapInt512FreeNode ?` | 🟡 | virtual | Frees a node pointer using alignment 4 through the shared memory manager. |
| `00cfbee0` | `HashListMapInt512ScalarDeletingDtor ?` | 🟡 | dtor | Restores the map vtable, removes all nodes, and frees the object if the delete flag's low bit is set. |
