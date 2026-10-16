# Class: SoeUtil::HashListMap<int,ClientTimers::TimerDefinition,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c3700` | `TimerDefinitionMapInit` | ✅ | ctor | Installs the timer-definition map vtable, zeroes its counters and 32-bucket storage, and returns the initialized map pointer. |
| `009c3bf0` | `TimerDefinitionNodeAlloc` | ✅ | function | If the current allocator manager exists, calls its allocation slot and exits; otherwise it tries the global manager and exits if that manager exists; the fallback request occurs only when both managers are absent (size … |
| `009c3c30` | `TimerDefinitionNodeFree` | ✅ | function | Calls the shared free routine for the supplied pointer with alignment 4, then returns. |
| `009c6120` | `DestroyTimerDefinitionHashList` | ✅ | dtor | Restores the timer-definition map vtable and removes all remaining head nodes through the type-specific node remover. |
| `009c61b0` | `— ⚠reject` | ⚪ | dtor | Removes all timer-definition nodes, frees the container only when the deleting flag is set, and otherwise returns its pointer. |
| `009c6270` | `DestroyTimerDefinitionHashListAlt` | ✅ | dtor | Restores the map vtable and repeatedly removes each remaining timer-definition node. |
| `009c6400` | `TimerDefinitionHashListInit` | ✅ | ctor | Initializes map counters and buckets, then installs the embedded HashList vtable and returns the object pointer. |
