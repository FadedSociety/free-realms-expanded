# Class: SoeUtil::Array<MerchantListEntry_const*,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cfde30` | `ArrayAllocateStorage` | ✅ | virtual | Returns existing storage when requested capacity is acceptable; otherwise computes a capacity, allocates capacity times four bytes at alignment four, and returns it with the chosen count. |
| `00cfded0` | `ArrayFreeStorage` | ✅ | virtual | Passes the supplied pointer to the shared allocator free path with alignment four. |
| `00cfdef0` | `ArrayDestroyOrSwapStorage` | ✅ | virtual | If the flag is set, calls the supplied object's virtual cleanup slot; otherwise swaps the data pointer and the two capacity/size dwords between the array and the supplied object. |
| `00cfe300` | `ArrayCleanupStorage` | ✅ | dtor | Restores the array vtable, clears the live count, frees the backing storage at alignment four, and nulls the storage pointer. |
| `00cfe620` | `ArrayScalarDeletingDtor` | ✅ | dtor | Restores the array vtable, clears its live count, frees and nulls its backing allocation, then frees the array object when deleting-flag bit 0 is set. |
