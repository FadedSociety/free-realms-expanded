# Class: SoeUtil::HashListMap<int,SoeUtil::IString<char>,128,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00986150` | `HashListMap128String_Constructor` | ✅ | ctor | Stores the map vtable, zeros its four scalar fields, clears 0x200 bytes of bucket storage at this+0x14, and returns this. |
| `009864e0` | `HashListMap128_AllocateNode` | ✅ | virtual | Requests memory from the current allocator when available, otherwise the global allocator; the fallback requests 0x20 bytes aligned to 4. |
| `00986520` | `HashListMap128_FreeNode` | ✅ | virtual | Forwards its object pointer and alignment 4 to 00401c00, then returns. |
| `00987220` | `HashListMap128_DestructorBody` | ✅ | dtor | Installs the map vtable and removes each head node through 00986a70 until the list is empty. |
| `009873d0` | `— ⚠reject` | ⚪ | dtor | Installs the map vtable, removes all nodes through 00986a70, then frees this object when flag bit 0 is set; otherwise returns this. |
| `00987410` | `HashListMap128_DestructorBody` | ✅ | dtor | Installs the map vtable, repeatedly removes the current head node with 00986a70, then restores the exception-list pointer and returns. |
| `00987690` | `HashListMap128_Constructor` | ✅ | ctor | Initializes map/list vtables, zeroes the count/head/tail fields, clears 0x200 bucket bytes, and returns this. |
