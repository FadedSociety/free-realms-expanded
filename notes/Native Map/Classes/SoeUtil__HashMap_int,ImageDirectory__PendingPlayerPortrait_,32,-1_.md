# Class: SoeUtil::HashMap<int,ImageDirectory::PendingPlayerPortrait*,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00987880` | `HashMap32PendingPortrait_Constructor` | ✅ | ctor | Stores the map vtable, zeroes its count and state, clears 0x80 bytes of bucket storage at this+0xc, and returns this. |
| `009880d0` | `HashMap32PendingPortrait_ClearThunk` | ✅ | dtor | Stores the pending-portrait map vtable then tail-jumps to 00987d70, the bucket cleanup routine. |
| `009880e0` | `HashMap32PendingPortrait_AllocateNode` | ✅ | virtual | Uses the current/global allocator sequence and requests 0xc bytes aligned to 4 as a fallback. |
| `00988120` | `HashMap32PendingPortrait_FreeNode` | ✅ | virtual | Loads the pointer argument, forwards it with alignment 4 to 00401c00, and returns with ret 4. |
| `00988460` | `HashMap32PendingPortrait_ScalarDeletingDtor` | ✅ | dtor | Installs the map vtable, clears its entries through 00987d70, frees the object when flag bit 0 is set, and otherwise returns this. |
| `00988f50` | `— ⚠reject` | ⚪ | dtor | Installs the pending-portrait map vtable and clears the buckets through 00987d70, restoring exception state afterward. |
| `00989100` | `HashMap32PendingPortrait_Constructor` | ✅ | ctor | Installs the map vtable, clears 0x80 bytes of bucket storage, zeros its state, then installs the Hash vtable and returns this. |
