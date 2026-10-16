# Class: SoeUtil::HashListMap<int,SoeGems::TextIndex<int>::InvertedList,16384,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b8b380` | `TextIndexInvertedListMap::Ctor` | ✅ | ctor | Installs the HashListMap vtable, zeros count and head fields, clears 0x10000 bytes of bucket storage, and initializes the remaining state to zero. |
| `00b93d40` | `InvertedListHashMapDtor` | ✅ | dtor | Restores the inverted-list map vtable, repeatedly removes and destroys the current entry until head offset 8 is zero, then returns. |
| `00b94600` | `InvertedListHashMapDeletingDtor` | ✅ | dtor | Restores the map vtable, removes and destroys entries, then frees this only when flag bit 0 is set. |
