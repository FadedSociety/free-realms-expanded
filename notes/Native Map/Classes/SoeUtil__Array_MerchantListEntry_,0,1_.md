# Class: SoeUtil::Array<MerchantListEntry*,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cfdf70` | `ArrayAllocateStorage` | ✅ | virtual | Returns existing storage when requested capacity is acceptable; otherwise computes a capacity, allocates capacity times four bytes at alignment four, and returns it with the chosen count. |
| `00cfe010` | `ArrayFreeStorage` | ✅ | virtual | Passes the supplied pointer to the shared allocator free path with alignment four. |
| `00cfe030` | `ArrayDestroyOrSwapStorage` | ✅ | virtual | If the flag is set, calls the supplied object's virtual cleanup slot; otherwise swaps the data pointer and the two capacity/size dwords between the array and the supplied object. |
| `00cfe330` | `ArrayCleanupStorage` | ✅ | dtor | Restores the array vtable, clears the live count, frees the backing storage at alignment four, and nulls the storage pointer. |
| `00cfe660` | `ArrayScalarDeletingDtor` | ✅ | dtor | Restores the array vtable, clears its live count, frees and nulls its backing allocation, then frees the array object when deleting-flag bit 0 is set. |
