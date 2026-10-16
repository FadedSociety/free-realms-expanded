# Class: SoeUtil::Array<Deep::FontFace*,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d006a0` | `ArrayAllocateStorage` | ✅ | virtual | Returns existing storage when capacity permits; otherwise allocates a four-byte pointer array sized to the requested element count and returns the new capacity. |
| `00d00740` | `ArrayFreeStorage` | ✅ | virtual | Frees the supplied array storage pointer through the shared allocator at alignment four. |
| `00d00760` | `ArrayDestroyOrSwapStorage ? ⚠rename` | 🟡 | virtual | If the flag is set, invokes the supplied object's virtual cleanup slot; otherwise swaps the data pointer and the two capacity/size fields between objects. |
| `00d009f0` | `ArrayCleanupStorage` | ✅ | dtor | Restores the array vtable, clears its live count, frees the backing storage at alignment four, and nulls the storage pointer. |
| `00d00d10` | `ArrayScalarDeletingDtor` | ✅ | dtor | Restores the array vtable, clears its live count, frees and nulls backing storage, then frees the array object when deleting-flag bit 0 is set. |
