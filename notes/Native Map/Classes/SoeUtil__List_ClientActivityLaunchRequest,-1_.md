# Class: SoeUtil::List<ClientActivityLaunchRequest,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c1480` | `— ⚠reject` | ⚪ | virtual | Runs the container allocation hook, uses the thread or global allocator when available, and otherwise requests 0xc0 bytes aligned to 8. |
| `009c14c0` | `FreeActivityLaunchRequestNode` | ✅ | virtual | Calls the shared allocator free routine with the node pointer and alignment value 8. |
| `009c1d00` | `DestroyActivityLaunchRequestList` | ✅ | dtor | Installs the typed list vtable and repeatedly unlinks and destroys the current head until the list is empty. |
| `009c1d30` | `ActivityLaunchRequestListScalarDeletingDtor` | ✅ | virtual | Installs the typed list vtable, destroys remaining nodes, and frees the allocation when delete flag bit 0 is set. |
