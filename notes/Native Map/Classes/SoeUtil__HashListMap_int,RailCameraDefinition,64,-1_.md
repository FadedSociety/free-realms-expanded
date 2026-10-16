# Class: SoeUtil::HashListMap<int,RailCameraDefinition,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bdac0` | `RailCameraDefinitionHashListMap64::Allocate` | ✅ | virtual | Allocates a 0x40-byte map entry aligned to 4 after trying the thread and global allocators. |
| `008bdb00` | `RailCameraDefinitionHashListMap64::FreeAllocation` | ✅ | virtual | Releases the supplied map entry with 4-byte alignment. |
| `008e0de0` | `HashListMapRailCameraDefinition_Dtor` | ✅ | dtor | Installs the map vtable and repeatedly destroys the node at map field +8 until the list is empty. |
| `008e2210` | `HashListMapRailCameraDefinition_ScalarDeletingDtor` | ✅ | dtor | Installs the map vtable, destroys linked nodes, and optionally frees the object when flag bit 0 is set. |
