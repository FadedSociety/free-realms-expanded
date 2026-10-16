# Class: SoeUtil::HashListMap<int,ActorCompositeEffectDefinition,256,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bc650` | `AllocateHashListStorage` | ✅ | virtual | Initializes allocator state, then allocates 0x50 bytes aligned to four through the thread manager, global manager, or aligned CRT fallback. |
| `008bc690` | `FreeHashListStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 4. |
| `0093c690` | `HashListMap::DtorBody → SoeUtil::HashListMap<int,ActorCompositeEffectDefinition…` | ✅ | dtor | Writes the HashListMap vtable, then repeatedly destroys the current node at offset 8 until no node remains. |
| `0093c740` | `HashListMap::ScalarDeletingDtor → SoeUtil::HashListMap<int,ActorCompositeEffect…` | ✅ | dtor | Writes the map vtable, destroys current nodes until empty, and frees the object when the deleting flag low bit is set. |
| `0093f7a0` | `DestroyActorCompositeEffectNodes` | ✅ | dtor | Restores the hash-list-map vtable, then repeatedly calls 00926720 on the node at element 2 until the head is empty. |
| `0093f810` | `InitActorCompositeEffectMap` | ✅ | function | Installs the map vtable, zeros count, head, tail, and its 0x400-byte bucket array, then switches to the HashList specialization vtable. |
