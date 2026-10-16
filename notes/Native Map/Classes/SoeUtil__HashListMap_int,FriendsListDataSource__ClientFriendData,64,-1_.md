# Class: SoeUtil::HashListMap<int,FriendsListDataSource::ClientFriendData,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cb2eb0` | `FriendsHashListMapCtor` | ✅ | ctor | Installs the hash-list-map vtable, clears list pointers/count, zeroes the 64-bucket table, and resets the remaining map field. |
| `00cb3290` | `FriendsHashNodeAllocate` | ✅ | virtual | Runs allocation initialization, requests memory from the active allocator when available, falls back to a global allocator, then requests 0xf0 bytes aligned to 8. |
| `00cb32d0` | `FriendsHashNodeFree` | ✅ | virtual | Frees the supplied pointer through the aligned allocator helper with alignment 8. |
| `00cb5060` | `FriendsHashListMapDtor` | ✅ | dtor | Restores the map vtable and repeatedly removes the current list head until all nodes are released. |
| `00cb5090` | `FriendsHashListMapScalarDeletingDtor` | ✅ | dtor | Destroys all linked entries and frees the map object when the low bit of the deleting flag is set. |
| `00cb50d0` | `FriendsHashListDtor` | ✅ | dtor | Restores the hash-list vtable and clears its nodes by repeatedly removing the current head under an exception frame. |
| `00cb5220` | `FriendsHashListCtor` | ✅ | ctor | Installs the map vtable, clears list and bucket counts, zeroes the 64-bucket table, and then installs the hash-list base vtable. |
