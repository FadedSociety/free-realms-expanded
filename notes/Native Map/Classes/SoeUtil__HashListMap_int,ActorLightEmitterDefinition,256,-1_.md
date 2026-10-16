# Class: SoeUtil::HashListMap<int,ActorLightEmitterDefinition,256,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bc570` | `AllocateHashListStorage` | ✅ | virtual | Initializes allocator state, then allocates 0xe0 bytes aligned to 0x10 through the thread manager, global manager, or aligned CRT fallback. |
| `008bc5b0` | `FreeHashListStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 0x10. |
| `00919700` | `SoeUtil::HashListMapActorLightEmitterDefinition256DtorBody` | ✅ | dtor | Destructor body, 39 bytes decoded: stores the HashListMap vtable, then while the chain at +8 is non-null calls FUN_008fbb60 as a thiscall on the container (mov ecx,esi) with the node pushed, re-reading +8 each time. |
| `0091a300` | `SoeUtil::HashListMapActorLightEmitterDefinition256ScalarDtor` | ✅ | dtor | Scalar deleting destructor with the body inlined: stores the HashListMap vtable, loops over the +8 chain calling FUN_008fbb60 (thiscall with the node pushed), then frees the object with _free when flag bit 0 is set. |
| `00920b60` | `HashListMap_ActorLightEmitterDefinition_Dtor` | ✅ | dtor | Destructor body: stores the HashListMap<int,ActorLightEmitterDefinition,256,-1> vtable at this, then repeatedly destroys the node at this+8 through 008fbb60 until the list head is null. |
| `00920e30` | `HashListMap_ActorLightEmitterDefinition_Ctor` | ✅ | ctor | Constructor: stores the HashListMap<int,ActorLightEmitterDefinition,256,-1> vtable at this, zeroes the words at +4, +8, +0xc and +0x10, memsets the 0x400-byte bucket array at +0x14 to zero, then stores the HashList<Acto… |
