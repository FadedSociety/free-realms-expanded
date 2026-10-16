# Class: SoeUtil::HashListMap<int,ActorCameraEffectDefinition,256,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bc7e0` | `AllocateHashListStorage` | ✅ | virtual | Initializes allocator state, then allocates 0xd0 bytes aligned to 0x10 through the thread manager, global manager, or aligned CRT fallback. |
| `008bc820` | `FreeHashListStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 0x10. |
| `00919730` | `SoeUtil::HashListMapActorCameraEffectDefinition256DtorBody` | ✅ | dtor | Destructor body, 39 bytes: stores the HashListMap vtable, then loops over the chain at +8 calling FUN_008fbc10 as a thiscall on the container (mov ecx,esi) with the node pushed. |
| `0091a340` | `SoeUtil::HashListMapActorCameraEffectDefinition256ScalarDtor` | ✅ | dtor | Scalar deleting destructor with the body inlined: stores the HashListMap vtable, loops over the +8 chain calling FUN_008fbc10 (thiscall with the node pushed), then frees the object with _free when flag bit 0 is set. |
| `00920bd0` | `HashListMap_ActorCameraEffectDefinition_Dtor` | ✅ | dtor | Destructor body: stores the HashListMap<int,ActorCameraEffectDefinition,256,-1> vtable at this, then repeatedly destroys the node at this+8 through 008fbc10 until the list head is null. |
| `00920eb0` | `HashListMap_ActorCameraEffectDefinition_Ctor` | ✅ | ctor | Constructor: stores the HashListMap<int,ActorCameraEffectDefinition,256,-1> vtable at this, zeroes the words at +4, +8, +0xc and +0x10, memsets the 0x400-byte bucket array at +0x14 to zero, then stores the HashList<Acto… |
