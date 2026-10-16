# Class: SoeUtil::HashListMap<int,int,40960,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00987bd0` | `HashListMap40960Int_Constructor` | ✅ | ctor | Stores the map vtable, zeroes its scalar state and clears 0x28000 bytes of bucket storage, then returns this. |
| `009883d0` | `HashListMap40960_AllocateNode` | ✅ | virtual | Uses the configured/global allocator sequence and requests 0x14 bytes aligned to 4 on the global allocation path. |
| `00988410` | `HashListMap40960_FreeNode` | ✅ | virtual | Forwards its pointer and alignment 4 to 00401c00, then returns. |
| `0098a120` | `HashListMap::Destructor` | ✅ | dtor | Installs the HashListMap vtable and tail-jumps to 00989ab0 to clear the hash-list entries. |
| `0098a250` | `HashListMap::ScalarDeletingDestructor` | ✅ | dtor | Installs the HashListMap vtable, runs 00989ab0, frees this only when deleting flag bit 0 is set, and otherwise returns this. |
| `0098ae20` | `HashListMap::Destructor` | ✅ | dtor | Installs the HashListMap vtable and clears its hash-list entries through 00989ab0. |
| `0098b570` | `HashListMap::Constructor` | ✅ | ctor | Initializes count and list pointers, zeros the 40960-entry bucket array with 0x28000 bytes, then installs the HashList vtable. |
