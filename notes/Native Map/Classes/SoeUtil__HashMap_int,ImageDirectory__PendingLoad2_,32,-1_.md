# Class: SoeUtil::HashMap<int,ImageDirectory::PendingLoad2*,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00987820` | `HashMap32PendingLoad2_Constructor` | ✅ | ctor | Stores the map vtable, zeroes its count, clears 0x80 bytes of bucket storage at this+0xc, zeroes the other state field, and returns this. |
| `00987fe0` | `HashMap32PendingLoad_ClearThunk` | ✅ | dtor | Stores the map vtable and tail-jumps to 00987ce0, the 32-bucket clear routine. |
| `00988070` | `HashMap32PendingLoad_AllocateNode` | ✅ | virtual | Uses the current/global allocator sequence and requests 0xc bytes aligned to 4 when falling back to the global allocator. |
| `009880b0` | `HashMap32PendingLoad_FreeNode` | ✅ | virtual | Loads its pointer argument, pushes alignment 4, calls 00401c00, and returns with ret 4. |
| `00988430` | `HashMap32PendingLoad_ScalarDeletingDtor` | ✅ | dtor | Installs the map vtable, clears its entries through 00987ce0, frees the object when flag bit 0 is set, and otherwise returns this. |
| `00988f00` | `HashMap32PendingLoad_DestructorBody` | ✅ | dtor | Installs the pending-load map vtable and clears the bucket entries through 00987ce0, restoring the exception-list pointer afterward. |
| `00989080` | `HashMap32PendingLoad_Constructor` | ✅ | ctor | Installs the map vtable, zeroes its count and state, clears 0x80 bytes of buckets, then installs the SoeUtil::Hash vtable and returns this. |
