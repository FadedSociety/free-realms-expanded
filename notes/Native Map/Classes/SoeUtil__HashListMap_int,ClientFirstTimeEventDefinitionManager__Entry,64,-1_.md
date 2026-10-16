# Class: SoeUtil::HashListMap<int,ClientFirstTimeEventDefinitionManager::Entry,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bda60` | `ClientFirstTimeEventHashListMap64::Allocate` | ✅ | virtual | Allocates a 0x24-byte map entry aligned to 4 after trying the thread and global allocators. |
| `008bdaa0` | `ClientFirstTimeEventHashListMap64::FreeAllocation` | ✅ | virtual | Releases the supplied map entry with 4-byte alignment. |
| `00919900` | `SoeUtil::HashListMapClientFirstTimeEventDefinitionManagerEntry64DtorBody` | ✅ | dtor | Destructor body, 39 bytes: stores the HashListMap vtable, then loops over the chain at +8 calling FUN_008fac10 as a thiscall on the container with the node pushed. |
| `0091a580` | `SoeUtil::HashListMapClientFirstTimeEventDefinitionManagerEntry64ScalarDtor` | ✅ | dtor | Scalar deleting destructor with the body inlined: stores the HashListMap vtable, loops over the +8 chain calling FUN_008fac10 (thiscall with the node pushed), then frees the object with _free when flag bit 0 is set. |
