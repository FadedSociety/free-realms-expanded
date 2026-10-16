# Class: SoeUtil::HashListMap<int,CommandProcessor::CommandEventHandlerEntry,1024,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009860e0` | `HashListMap1024CommandEntry_Constructor` | ✅ | ctor | Initializes the map vtable and its four scalar fields to zero, clears 0x1000 bytes starting at the bucket array at this+0x14, and returns this. |
| `00986470` | `HashListMap1024_AllocateNode` | ✅ | virtual | Requests memory from the current allocator when available, otherwise the global allocator; the fallback requests 0x18 bytes aligned to 4, matching the command-entry node size. |
| `009864b0` | `HashListMap1024_FreeNode ⚠downgrade` | 🟢 | virtual | Forwards its object pointer and alignment 4 to 00401c00, then returns. |
| `00986890` | `HashListMap1024CommandEntry_DestructorBody` | ✅ | dtor | Installs the map vtable, then repeatedly removes the node at this+8 through 009863c0 until the head is null. |
| `009868c0` | `— ⚠reject` | ⚪ | dtor | Installs the map vtable, removes all nodes through 009863c0, then frees this object when flag bit 0 is set; otherwise returns this. |
| `00986960` | `HashListMap1024CommandEntry_DestructorBody` | ✅ | dtor | Installs the map vtable and repeatedly removes its current head node through 009863c0 until the map is empty. |
| `009869d0` | `HashListMap1024CommandEntry_Constructor` | ✅ | ctor | Initializes the map vtable and zeroes the size/head/tail state and 1024 bucket slots; then installs the HashList vtable and returns this. |
