# Class: SoeUtil::HashListMap<int,FrictionMaterialTableEntryConfig,10,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009d31b0` | `FrictionMaterialHashMap::NodeAlloc` | ✅ | virtual | Runs allocator setup, tries the current allocator and fallback allocator, then requests a 0x20-byte node aligned to four bytes. |
| `009d31f0` | `FrictionMaterialHashMap::NodeFree` | ✅ | virtual | Frees the supplied node pointer through FUN_00401c00 with four-byte alignment. |
