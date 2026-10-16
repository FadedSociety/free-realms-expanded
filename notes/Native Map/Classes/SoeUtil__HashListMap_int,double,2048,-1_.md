# Class: SoeUtil::HashListMap<int,double,2048,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bc350` | `AllocateHashListStorage` | ✅ | virtual | Initializes allocator state, then allocates 0x18 bytes aligned to eight through the thread manager, global manager, or aligned CRT fallback. |
| `008bc390` | `FreeHashListStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 8. |
| `008e08b0` | `HashListMapIntDouble2048_Dtor` | ✅ | dtor | Installs the map vtable and destroys linked nodes from the map head until empty. |
| `008e2000` | `HashListMapIntDouble2048_ScalarDeletingDtor` | ✅ | dtor | Installs the map vtable, destroys linked nodes until empty, and frees the object when flag bit 0 is set. |
