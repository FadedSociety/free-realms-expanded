# Class: SoeUtil::HashListMap<int,ClientAccountManager::WebServiceError,16,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00706500` | `HashListMapConstructor ?` | 🟡 | ctor | Initializes the hash-list map count and clears its bucket storage. |
| `00709090` | `HashListMapAlloc` | ✅ | virtual | Uses the current memory manager or global fallback to allocate 0x24 bytes aligned to 4. |
| `007090d0` | `HashListMapFree` | ✅ | virtual | Passes the supplied pointer and alignment 4 to the memory-manager free routine. |
| `00711770` | `WebServiceErrorHashMapDestructor` | ✅ | dtor | Installs hash-list-map vtable and removes entries until list is empty. |
| `00711840` | `WebServiceErrorHashMapScalarDeletingDtor` | ✅ | dtor | Removes hash-list entries, returns the original object when deletion flag bit 0 is clear, and frees it when that bit is set. |
| `007119f0` | `WebServiceErrorHashMapDestructor` | ✅ | dtor | Installs map vtable and repeatedly removes list head until empty. |
| `00711ee0` | `WebServiceErrorHashMapCtor` | ✅ | ctor | Temporarily installs the HashListMap vtable, zeros root/count fields and sixteen bucket heads, then installs the HashList vtable as the final vptr and returns this. |
