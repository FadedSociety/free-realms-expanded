# Class: SoeUtil::HashListMap<int,IgnoreData,64,16>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cf8f90` | `— ⚠reject` | ⚪ | dtor | Restores typed map vtables, destroys linked nodes, frees the auxiliary tree, then restores the base map vtable and destroys remaining nodes. |
| `00cf9a20` | `— ⚠reject` | ⚪ | virtual | Subtracts 0x114 from the this pointer, then jumps to 00cf9940, which removes and returns a free-list head entry. |
| `00cf9a30` | `— ⚠reject` | ⚪ | virtual | Subtracts 0x114 from the this pointer, then jumps to 00cf99b0, which pushes the entry onto the free-list head. |
| `00cf9a40` | `HashListMapIgnoreDataScalarDeletingDtor ?` | 🟡 | dtor | Calls the map cleanup and frees the object when the delete flag's low bit is set. |
| `00cf9a60` | `HashListMapIgnoreDataPooledInit ?` | 🟡 | init | Initializes the 64-bucket base map, replaces the vtable with the fixed-block specialization, and initializes an embedded four-block list pool. |
