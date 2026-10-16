# Class: SoeUtil::HashList<BaseItemSearchEngine::SearchEntry,1024,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c305a0` | `SearchEntryHashListCtor` | ✅ | ctor | Initializes counters, clears 0x1000 bytes of bucket heads, then installs the SearchEntry HashList vtable after its map base. |
| `00c30620` | `SearchEntryHashListScalarDeletingDtor` | ✅ | dtor | Calls the map cleanup body and frees the list object when flag bit 0 is set. |
