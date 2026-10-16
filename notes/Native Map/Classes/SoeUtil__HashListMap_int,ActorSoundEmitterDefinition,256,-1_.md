# Class: SoeUtil::HashListMap<int,ActorSoundEmitterDefinition,256,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bd5c0` | `ActorSoundEmitterDefinitionHashListMap256::AllocateNode` | ✅ | virtual | Tries the thread and global allocators, then allocates 0x1c0 bytes aligned to 0x10. |
| `008bd600` | `ActorSoundEmitterDefinitionHashListMap256::FreeNode` | ✅ | virtual | Calls FreeMemory with the supplied pointer and 0x10-byte alignment. |
| `00940b70` | `DestroyActorSoundEmitterMap` | ✅ | dtor | Restores the map vtable, then repeatedly calls 007e60e0 on the current head node until the list is empty. |
| `00940c10` | `— ⚠reject` | ⚪ | dtor | Drains map nodes through 007e60e0, returns the object if the deleting flag is clear, otherwise frees it. |
| `00941500` | `DestroyActorSoundEmitterMap` | ✅ | dtor | Restores map vtable, then repeatedly calls 007e60e0 on the current head node until empty. |
| `009415d0` | `InitActorSoundEmitterHashMap` | ✅ | ctor | Installs map vtable, zeros head/tail/count and 256 buckets, then installs HashList specialization vtable. |
