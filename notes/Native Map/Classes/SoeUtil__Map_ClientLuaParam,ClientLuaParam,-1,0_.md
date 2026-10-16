# Class: SoeUtil::Map<ClientLuaParam,ClientLuaParam,-1,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00708a80` | `MapAlloc` | ✅ | virtual | Initializes the allocation path, checks the current memory manager and global fallback, then requests 0x150 bytes aligned to 8. |
| `00708ac0` | `MapFree` | 🟢 | virtual | Passes the supplied pointer and alignment 8 to the memory-manager free routine. |
| `0070ece0` | `ClientLuaParamMapDestructor` | ✅ | dtor | Installs map vtable, releases owned storage, invokes cleanup, then clears map data and count. |
| `0070ef30` | `ClientLuaParamMapScalarDeletingDtor` | ✅ | dtor | Releases map storage; returns the original object when (param2 & 1) == 0 and frees it when bit 0 is set. |
| `00711740` | `ClientLuaParamMapCtor` | ✅ | ctor | Zeros root/count, installs map vtable, and copies supplied contents through shared assignment helper. |
