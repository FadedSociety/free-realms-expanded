# Class: SoeUtil::HashListMap<int,SoeUtil::StringFixed<32>,2048,1024>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d080f0` | `StringHashListMap_Clear` | ✅ | dtor | Clears list and map, releases both mapped allocations, switches to base HashListMap vtable, and clears its nodes. |
| `00d08b60` | `StringHashListMap::ScalarDeletingDtor` | ✅ | virtual | Cleans up the map, frees object when deleting flag is set, and otherwise returns original pointer. |
| `00d08b80` | `StringHashListMap_Init` | ✅ | ctor | Zeros map counters and buckets, installs map vtable, and calls base initializer. |
