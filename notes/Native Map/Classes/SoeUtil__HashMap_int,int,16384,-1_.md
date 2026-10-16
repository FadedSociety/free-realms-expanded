# Class: SoeUtil::HashMap<int,int,16384,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009878b0` | `HashMap16384Int_Constructor` | ✅ | ctor | Stores the map vtable, zeroes the count and state, clears 0x10000 bytes of bucket storage at this+0xc, and returns this. |
| `00988140` | `HashMap16384Int_ClearThunk` | ✅ | dtor | Writes the map vtable into [this] then tail-jumps to 009878e0 to release the bucket contents. |
| `00988150` | `HashMap16384Int_AllocateNode` | ✅ | virtual | Uses the allocator vtable/global fallbacks and requests 0xc bytes aligned to 4 when no custom allocator is available. |
| `00988190` | `HashMap16384Int_FreeNode` | ✅ | virtual | Forwards its pointer argument and alignment 4 to 00401c00, then returns. |
| `00988490` | `HashMap16384Int_ScalarDeletingDtor` | ✅ | dtor | Installs the map vtable, clears all buckets through 009878e0, optionally frees the object when flag bit 0 is set, and otherwise returns this. |
