# Class: SoeUtil::HashListMap<int,int,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c6f80` | `IntMap64Init` | ✅ | ctor | Installs the int-map vtable, zeros its counters, clears its 64-bucket array, and returns the map pointer. |
| `009c75f0` | `IntegerMap64NodeAlloc` | ✅ | function | If the current allocator manager exists, calls its allocation slot and exits; otherwise it tries the global manager and exits if that manager exists; the fallback request occurs only when both managers are absent (size … |
| `009c7630` | `IntegerMap64NodeFree` | ✅ | function | Calls the shared free routine with alignment 4 for the supplied integer-map node and returns. |
| `009c8280` | `DestroyMysteryChestIntegerMap` | ✅ | dtor | Restores the integer map vtable and removes remaining head nodes through the integer map unlink routine. |
| `009c82f0` | `— ⚠reject` | ⚪ | dtor | Removes all integer-map nodes, conditionally frees the container, and otherwise returns its pointer. |
| `009c8580` | `DestroyIntegerMap64` | ✅ | dtor | Restores the integer-map vtable and repeatedly removes each remaining head node through the type-specific unlink routine. |
| `009c8720` | `IntegerHashList64Init` | ✅ | ctor | Initializes map counters and its 64 buckets, then installs the embedded HashList vtable. |
