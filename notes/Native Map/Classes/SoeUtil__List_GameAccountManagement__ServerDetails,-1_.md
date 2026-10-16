# Class: SoeUtil::List<GameAccountManagement::ServerDetails,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00708840` | `ListAlloc` | ✅ | virtual | Initializes the list allocator path, asks the current memory manager for storage, falls back to the global manager, then requests 0x34 bytes aligned to 4. |
| `00708880` | `ListFree` | ✅ | virtual | Passes the supplied pointer and alignment 4 to the memory-manager free routine. |
| `00712020` | `ListServerDetails_DtorBody` | ✅ | dtor | Stores SoeUtil::List<GameAccountManagement::ServerDetails,-1>::vftable at [this]; while [this+4] is non-null calls 0070e5d0 with the head node as argument (ECX=this) to unlink and release it. Returns void; does not free… |
