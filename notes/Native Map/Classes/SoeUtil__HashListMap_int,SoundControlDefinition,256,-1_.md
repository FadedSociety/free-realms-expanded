# Class: SoeUtil::HashListMap<int,SoundControlDefinition,256,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bd740` | `SoundControlDefinitionHashListMap256::Allocate` | ✅ | virtual | Allocates 0xe0 bytes aligned to 0x10 after trying the thread and global allocators. |
| `008bd780` | `SoundControlDefinitionHashListMap256::FreeAllocation` | ✅ | virtual | Releases the supplied map allocation with 0x10-byte alignment. |
| `009198a0` | `SoeUtil::HashListMapSoundControlDefinition256DtorBody` | ✅ | dtor | Destructor body, 39 bytes: stores the HashListMap vtable, then loops over the chain at +8 calling FUN_008fbff0 as a thiscall on the container with the node pushed. |
| `0091a500` | `SoeUtil::HashListMapSoundControlDefinition256ScalarDtor` | ✅ | dtor | Scalar deleting destructor with the body inlined: stores the HashListMap vtable, loops over the +8 chain calling FUN_008fbff0 (thiscall with the node pushed), then frees the object with _free when flag bit 0 is set. |
| `00920d60` | `HashListMap_SoundControlDefinition_Dtor` | ✅ | dtor | Destructor body: stores the HashListMap<int,SoundControlDefinition,256,-1> vtable at this, then repeatedly destroys the node at this+8 through 008fbff0 until the list head is null. |
| `009210b0` | `HashListMap_SoundControlDefinition_Ctor` | ✅ | ctor | Constructor: stores the HashListMap<int,SoundControlDefinition,256,-1> vtable at this, zeroes the words at +4, +8, +0xc and +0x10, memsets the 0x400-byte bucket array at +0x14 to zero, then stores the HashList<SoundCont… |
