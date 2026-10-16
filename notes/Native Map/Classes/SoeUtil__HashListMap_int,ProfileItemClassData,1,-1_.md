# Class: SoeUtil::HashListMap<int,ProfileItemClassData,1,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bbcc0` | `AllocateHashListStorage` | ✅ | virtual | Initializes allocator state, then allocates 0x18 bytes aligned to four through the thread manager, global manager, or aligned CRT fallback. |
| `008bbd00` | `FreeHashListStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 4. |
| `008e05e0` | `HashListMapProfileItemClassData_Dtor` | ✅ | dtor | Installs the map vtable and repeatedly destroys the node at map field +8 until the map is empty. |
| `008e1f90` | `HashListMapProfileItemClassData_ScalarDeletingDtor` | ✅ | dtor | Installs the map vtable, repeatedly destroys linked map nodes, and frees the object when flag bit 0 is set. |
| `008ea780` | `HashListMapDtor → SoeUtil::HashListMap<int,ProfileItemClassData,1,-1>::Destruct…` | ✅ | dtor | Installs map vtable and destroys current head nodes until map is empty. |
| `008f15b0` | `HashListMapProfileItemClassData_Ctor` | ✅ | ctor | Installs the ProfileItemClassData hash-map vtable, zeros its five state fields, then leaves the HashList base vtable installed. |
