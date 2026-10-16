# Class: SoeUtil::HashListMap<int,ClientActivityLaunchManager::ClientActivityLaunchProcessorStub,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c0e00` | `ClientActivityLaunchProcessorMapCtor` | ✅ | ctor | Installs the typed hash-map vtable, clears map counters and pointers, zeroes 32 bucket entries, and returns this. |
| `009c1410` | `— ⚠reject` | ⚪ | virtual | Runs the container allocation hook, uses the thread or global allocator when available, and otherwise requests 0x50 bytes aligned to 4. |
| `009c1450` | `FreeProcessorHashNode` | ✅ | virtual | Calls the shared allocator free routine with the pointer and alignment value 4. |
| `009c2b50` | `DestroyActivityLaunchProcessorHashMap` | ✅ | dtor | Installs the typed map vtable and repeatedly removes the current node until the map is empty. |
| `009c2b80` | `ActivityLaunchProcessorHashMapScalarDeletingDtor` | ✅ | virtual | Installs the typed map vtable, removes all nodes, then frees the allocation when delete flag bit 0 is set. |
| `009c2c50` | `DestroyActivityLaunchProcessorMap` | ✅ | dtor | Installs the typed map vtable and removes every node through the shared map-node cleanup routine. |
| `009c2db0` | `ActivityLaunchProcessorMapInit` | ✅ | ctor | Sets the processor map vtable, clears its counters and bucket array, then installs the embedded hash-list vtable. |
