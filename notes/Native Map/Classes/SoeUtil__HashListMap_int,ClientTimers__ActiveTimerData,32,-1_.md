# Class: SoeUtil::HashListMap<int,ClientTimers::ActiveTimerData,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c37e0` | `ActiveTimerDataMapInit` | ✅ | ctor | Installs the active-timer map vtable, zeroes its counters and 32-bucket storage, and returns the initialized map pointer. |
| `009c3d00` | `ActiveTimerDataNodeAlloc` | ✅ | function | If the current allocator manager exists, calls its allocation slot and exits; otherwise it tries the global manager and exits if that manager exists; the fallback request occurs only when both managers are absent (size … |
| `009c3d40` | `ActiveTimerDataNodeFree` | ✅ | function | Calls the shared free routine for the supplied pointer with alignment 4, then returns. |
| `009c6180` | `DestroyActiveTimerHashList` | ✅ | dtor | Restores the active-timer map vtable and removes all remaining head nodes through the type-specific node remover. |
| `009c6230` | `— ⚠reject` | ⚪ | dtor | Removes all active-timer nodes, frees the container only when the deleting flag is set, and otherwise returns its pointer. |
| `009c6350` | `DestroyActiveTimerHashListAlt` | ✅ | dtor | Restores the active-timer map vtable and repeatedly removes each remaining node. |
| `009c6500` | `ActiveTimerHashListInit` | ✅ | ctor | Initializes map counters and buckets, then installs the embedded HashList vtable and returns the object pointer. |
