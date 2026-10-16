# Class: SoeUtil::HashListMap<int,ActorRumbleEffectDefinition,256,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bc8a0` | `AllocateHashListStorage` | ✅ | virtual | Initializes allocator state, then allocates 0xd0 bytes aligned to 0x10 through the thread manager, global manager, or aligned CRT fallback. |
| `008bc8e0` | `FreeHashListStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 0x10. |
| `00919760` | `SoeUtil::HashListMapActorRumbleEffectDefinition256DtorBody` | ✅ | dtor | Destructor body, 39 bytes: stores the HashListMap vtable, then loops over the chain at +8 calling FUN_008fbcc0 as a thiscall on the container with the node pushed. |
| `0091a380` | `SoeUtil::HashListMapActorRumbleEffectDefinition256ScalarDtor` | ✅ | dtor | Scalar deleting destructor with the body inlined: stores the HashListMap vtable, loops over the +8 chain calling FUN_008fbcc0 (thiscall with the node pushed), then frees the object with _free when flag bit 0 is set. |
| `00920c40` | `HashListMap_ActorRumbleEffectDefinition_Dtor` | ✅ | dtor | Destructor body: stores the HashListMap<int,ActorRumbleEffectDefinition,256,-1> vtable at this, then repeatedly destroys the node at this+8 through 008fbcc0 until the list head is null. |
| `00920f30` | `HashListMap_ActorRumbleEffectDefinition_Ctor` | ✅ | ctor | Constructor: stores the HashListMap<int,ActorRumbleEffectDefinition,256,-1> vtable at this, zeroes the words at +4, +8, +0xc and +0x10, memsets the 0x400-byte bucket array at +0x14 to zero, then stores the HashList<Acto… |
