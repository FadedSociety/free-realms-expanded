# Class: SoeUtil::HashListMap<int,BaseItemSearchEngine::SearchEntry,1024,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c2dad0` | `SoeUtil::HashListMap::HashListMap ⚠downgrade` | 🟢 | ctor | Constructor of the 1024-bucket hash list map: installs SoeUtil::HashListMap<int,BaseItemSearchEngine::SearchEntry,1024,-1>::vftable, zeroes the count/head fields at +4..+16 and memsets the bucket array at +0x14 to 0x100… |
| `00c2e3b0` | `HashListAllocator` | ✅ | method | Initializes allocator state, tries thread-local then global memory managers, then allocates 0xf0 bytes at alignment 4. |
| `00c2e3f0` | `HashListFreeBuffer` | ✅ | method | Passes its pointer argument to FUN_00401c00 with four-byte alignment. |
| `00c2fe60` | `RemoveSearchEntry` | 🟢 | function | Unlinks an entry from its key bucket and doubly linked list, decrements count, destroys a ScoreCard array member, then calls the map's virtual disposer. |
| `00c304b0` | `SearchEntryHashMapClear` | ✅ | dtor | Restores the map vtable, then removes entries from the head through FUN_00c2fe60 until empty. |
| `00c304f0` | `SearchEntryHashMapScalarDeletingDtor` | ✅ | dtor | Restores the map vtable, removes all entries, and frees the object when flag bit 0 is set. |
| `00c30530` | `SearchEntryHashMapClear` | ✅ | dtor | Restores the SearchEntry map vtable and repeatedly disposes the current head through FUN_00c2fe60. |
