# Class: SoeUtil::HashListMap<unsigned_int,SoeGems::TextIndex<int>::Document,0,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b93770` | `DocumentHashMapCtor` | ✅ | ctor | Installs base then inline-map vtables, zeros hash-list fields and counters, clears embedded map pointers, and disables the auxiliary flag. |
| `00b937e0` | `Thunk_ThisPlus18_00731980` | ✅ | virtual | Adjusts ECX by 0x18 and jumps to 00731980. |
| `00b937f0` | `Thunk_ThisPlus18_007319f0` | ✅ | virtual | Adjusts ECX by 0x18 and jumps to 007319f0. |
| `00b93c20` | `DocumentHashMapDtorInline` | ✅ | dtor | Restores the inline-map vtable, removes and cleans all hash-list entries, frees embedded tree nodes and resets their fields, then calls the base-map destructor. |
| `00b93ce0` | `DocumentHashMapInlineDeletingDtor` | ✅ | dtor | Calls the inline-map destructor, frees this only when flag bit 0 is set, and returns this otherwise. |
