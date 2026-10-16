# Class: SoeUtil::HashMap<int,SoeUtil::IString<char>,16384,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00987a30` | `HashMap16384String_Constructor` | ✅ | ctor | Stores the map vtable, zeros the count/state, clears 0x10000 bytes of bucket storage, and returns this. |
| `00988220` | `HashMap16384String_AllocateNode` | ✅ | virtual | Uses the allocator vtable/global fallback sequence; the global allocator fallback requests 0x18 bytes aligned to 4. |
| `00988260` | `HashMap16384String_FreeNode` | ✅ | virtual | Forwards its pointer and alignment 4 to 00401c00, then returns. |
| `0098b7d0` | `HashMap16384String::Destructor` | ✅ | dtor | Installs the map vtable and clears all 16384 bucket chains, releasing each node's IString value and freeing nodes. |
| `0098b990` | `HashMap16384String::ScalarDeletingDestructor` | ✅ | dtor | Calls the 16384-bucket string-map destructor; frees this only when deleting flag bit 0 is set, otherwise returns this. |
