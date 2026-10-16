# Class: SoeUtil::HashListMap<int,MinimapIndicatorEntry,64,32>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a90810` | `MinimapIndicatorHashMapDestructor` | ✅ | dtor | Installs the 32-inline map vtable, removes all hash-list entries, recursively frees the two search-tree branches and their storage, then clears tree pointers and restores the no-inline map vtable. |
| `00a91340` | `MinimapIndicatorHashMapDeletingDestructor` | ✅ | dtor | Calls the map cleanup body, frees the object only when flag bit 0 is set, and otherwise returns the original pointer. |
| `00a91380` | `MinimapIndicatorHashMapConstructor` | ✅ | ctor | Zeros the map counters and 64-bucket array, installs the 32-inline specialization vtable, and initializes its pool free-list sentinel. |
