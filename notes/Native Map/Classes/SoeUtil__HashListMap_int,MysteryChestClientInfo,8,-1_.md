# Class: SoeUtil::HashListMap<int,MysteryChestClientInfo,8,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c7540` | `MysteryChestInfoNodeAlloc` | ✅ | function | If the current allocator manager exists, calls its allocation slot and exits; otherwise it tries the global manager and exits if that manager exists; the fallback request occurs only when both managers are absent (size … |
| `009c7580` | `MysteryChestInfoNodeFree` | ✅ | function | Calls the common free routine on the supplied pointer with alignment 4 and returns. |
| `009c8250` | `DestroyMysteryChestMap` | ✅ | dtor | Restores the Mystery Chest info map vtable and removes remaining head nodes through the type-specific unlink routine. |
| `009c82b0` | `— ⚠reject` | ⚪ | dtor | Removes all Mystery Chest info nodes, conditionally frees the container, and otherwise returns its pointer. |
| `009c84b0` | `DestroyMysteryChestHashListAlt` | ✅ | dtor | Restores the info map vtable and repeatedly unlinks each remaining node from the map. |
| `009c86a0` | `MysteryChestInfoHashListInit` | ✅ | ctor | Zeros list counters, heads, tails and eight bucket entries, then installs the embedded HashList vtable. |
