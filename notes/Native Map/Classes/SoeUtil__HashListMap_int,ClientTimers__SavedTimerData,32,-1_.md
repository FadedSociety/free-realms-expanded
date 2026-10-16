# Class: SoeUtil::HashListMap<int,ClientTimers::SavedTimerData,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c3770` | `SavedTimerDataMapInit` | ✅ | ctor | Installs the saved-timer map vtable, zeroes its counters and 32-bucket storage, and returns the initialized map pointer. |
| `009c3c90` | `SavedTimerDataNodeAlloc` | ✅ | function | If the current allocator manager exists, calls its allocation slot and exits; otherwise it tries the global manager and exits if that manager exists; the fallback request occurs only when both managers are absent (size … |
| `009c3cd0` | `SavedTimerDataNodeFree` | ✅ | function | Calls the shared free routine for the supplied pointer with alignment 8, then returns. |
| `009c6150` | `DestroySavedTimerHashList` | ✅ | dtor | Restores the saved-timer map vtable and removes all remaining head nodes through the type-specific node remover. |
| `009c61f0` | `— ⚠reject` | ⚪ | dtor | Removes all saved-timer nodes, frees the container only when the deleting flag is set, and otherwise returns its pointer. |
| `009c62e0` | `DestroySavedTimerHashListAlt` | ✅ | dtor | Restores the saved-timer map vtable and repeatedly removes each remaining node. |
| `009c6480` | `SavedTimerHashListInit` | ✅ | ctor | Initializes map counters and buckets, then installs the embedded HashList vtable and returns the object pointer. |
