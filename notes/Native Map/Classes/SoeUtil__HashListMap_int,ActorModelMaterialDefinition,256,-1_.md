# Class: SoeUtil::HashListMap<int,ActorModelMaterialDefinition,256,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bc720` | `AllocateHashListStorage` | ✅ | virtual | Initializes allocator state, then allocates 0x70 bytes aligned to four through the thread manager, global manager, or aligned CRT fallback. |
| `008bc760` | `FreeHashListStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 4. |
| `00928ed0` | `DrainActorModelMaterialDefinitions` | ✅ | dtor | Installs the hash map vtable and unlinks nodes from hash/list storage until its list head is empty. |
| `00929890` | `HashListMapScalarDeletingDtor → SoeUtil::HashListMapActorModelMaterialDefinitio…` | 🟢 | dtor | Unlinks each hash/list node and frees the map only when the deleting flag is set. |
| `00935270` | `HashListMap::DestroyNodes` | ✅ | dtor | Installs the HashListMap vtable, then repeatedly unlinks the current node at this+8 until the node pointer becomes null. |
| `009352e0` | `HashListMap::Initialize` | ✅ | ctor | Installs the HashListMap vtable, zeroes the list heads and 256-bucket table, clears the counters, then installs the HashList vtable. |
