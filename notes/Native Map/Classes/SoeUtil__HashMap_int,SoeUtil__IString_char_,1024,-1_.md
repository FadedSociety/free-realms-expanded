# Class: SoeUtil::HashMap<int,SoeUtil::IString<char>,1024,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00987a90` | `HashMap1024String_Constructor` | ✅ | ctor | Stores the map vtable, zeroes state fields, clears 0x1000 bytes of bucket storage, and returns this. |
| `00988280` | `HashMap1024String_AllocateNode` | ✅ | virtual | Uses the allocator vtable/global fallbacks; the fallback requests 0x18 bytes aligned to 4. |
| `009882c0` | `HashMap1024String_FreeNode` | ✅ | virtual | Forwards its pointer and alignment 4 to 00401c00, then returns. |
| `0098b7e0` | `HashMap1024String::Destructor` | ✅ | dtor | Installs the map vtable and clears all 1024 bucket chains, releasing each node's IString value and freeing nodes. |
| `0098b9c0` | `HashMap1024String::ScalarDeletingDestructor` | ✅ | dtor | Calls the 1024-bucket string-map destructor; frees this only when deleting flag bit 0 is set, otherwise returns this. |
