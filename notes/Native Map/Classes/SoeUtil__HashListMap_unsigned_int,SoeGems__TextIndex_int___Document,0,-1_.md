# Class: SoeUtil::HashListMap<unsigned_int,SoeGems::TextIndex<int>::Document,0,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b93680` | `DocumentHashMapDtor` | ✅ | dtor | Restores the base-map vtable, repeatedly removes and cleans the current entry, then frees bucket storage at object offset 0x14. |
| `00b93750` | `DocumentHashMapDeletingDtor` | ✅ | dtor | Calls the document-map destructor, frees this only when flag bit 0 is set, and returns this otherwise. |
