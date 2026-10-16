# Class: SoeUtil::HashListMap<int,AssetManagement::AssetRequestInternal::AssetReadyHandler::HashEntry,1024,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bd260` | `AssetReadyHandlerHashListMap1024::AllocateNode` | ✅ | virtual | Initializes allocator state, tries the thread and global allocators, then allocates 0x28 bytes aligned to 4. |
| `008bd2a0` | `AssetReadyHandlerHashListMap1024::FreeAllocation` | ✅ | virtual | Calls FreeMemory with the supplied pointer and 4-byte alignment, then returns. |
| `00919840` | `SoeUtil::HashListMapAssetReadyHandlerHashEntry1024DtorBody` | ✅ | dtor | Destructor body, 39 bytes: stores the HashListMap vtable, then loops over the chain at +8 calling FUN_008fa980 as a thiscall on the container with the node pushed. |
| `0091a480` | `SoeUtil::HashListMapAssetReadyHandlerHashEntry1024ScalarDtor` | ✅ | dtor | Scalar deleting destructor with the body inlined: stores the HashListMap vtable, loops over the +8 chain calling FUN_008fa980 (thiscall with the node pushed), then frees the object with _free when flag bit 0 is set. |
