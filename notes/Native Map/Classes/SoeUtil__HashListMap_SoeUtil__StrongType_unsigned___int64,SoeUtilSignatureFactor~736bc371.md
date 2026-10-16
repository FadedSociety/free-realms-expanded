# Class: SoeUtil::HashListMap<SoeUtil::StrongType<unsigned___int64,SoeUtilSignatureFactoryFoundationGuid,0>,FactoryFoundation,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009e3490` | `FactoryFoundationMapDtor` | ✅ | dtor | Sets map vtable and destroys head nodes until empty. |
| `009e34c0` | `FactoryFoundationMapAllocateNode` | ✅ | virtual | Uses current/global allocator or requests 0x118 bytes aligned to 8 as fallback. |
| `009e3500` | `FactoryFoundationMapFreeNode` | ✅ | virtual | Passes pointer and alignment 8 to shared allocator free helper. |
| `009e3600` | `FactoryFoundationMapScalarDeletingDtor` | ✅ | dtor | Destroys map nodes and frees this when deleting flag is set. |
