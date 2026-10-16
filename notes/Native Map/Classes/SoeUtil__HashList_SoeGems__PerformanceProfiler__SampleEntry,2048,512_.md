# Class: SoeUtil::HashList<SoeGems::PerformanceProfiler::SampleEntry,2048,512>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d07d50` | `HashListSampleMap_Init` | ✅ | ctor | Initializes 2048-bucket storage, calls the map base initializer, then installs the HashList vtable. |
| `00d07d90` | `HashListSampleMap::ScalarDeletingDtor` | ✅ | virtual | Calls map cleanup, frees the object when deleting flag is set, and otherwise returns the original pointer. |
| `00d07dd0` | `HashListSampleMap_InitEmbedded` | ✅ | ctor | Initializes an embedded 2048-bucket hash list at this+8 and zeros a second 2048-entry bucket array at this+0xf040. |
