# Class: SoeUtil::HashListMap<int,CodeStringMappings::StringInfo,256,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bdfb0` | `CodeStringMappingHashListMap256::Allocate` | ✅ | virtual | Allocates a 0x24-byte map entry aligned to 4 after trying the thread and global allocators. |
| `008bdff0` | `CodeStringMappingHashListMap256::FreeAllocation` | ✅ | virtual | Releases the supplied map entry with 4-byte alignment. |
| `00919930` | `SoeUtil::HashListMapCodeStringMappingsStringInfo256DtorBody` | ✅ | dtor | Destructor body, 39 bytes: stores the HashListMap vtable, then loops over the chain at +8 calling FUN_008facc0 as a thiscall on the container with the node pushed. |
| `0091a5c0` | `SoeUtil::HashListMapCodeStringMappingsStringInfo256ScalarDtor` | ✅ | dtor | Scalar deleting destructor with the body inlined: stores the HashListMap vtable, loops over the +8 chain calling FUN_008facc0 (thiscall with the node pushed), then frees the object with _free when flag bit 0 is set. |
