# Class: SoeUtil::Array<PendingNotificationImage,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0095b4a0` | `— ⚠reject` | ⚪ | virtual | Chooses a resized capacity from the current count and request, returns existing storage when permitted, otherwise allocates capacity*0xc bytes. |
| `0095b540` | `ArrayFreeStorage` | ✅ | virtual | Calls FUN_00401c00 on the supplied pointer with alignment 4, then returns. |
| `0095b560` | `ArraySwapStorage` | ✅ | virtual | For a nonzero mode calls an indirect operation; otherwise exchanges the data pointer, size and capacity fields with param_2. |
| `0095e810` | `ArrayDestructor → SoeUtil::Array<PendingNotificationImage,0,1>::Destructor` | ✅ | dtor | Restores the array vtable, clears its size, frees backing storage with alignment 4, and clears the pointer. |
| `0095f240` | `Array::ScalarDeletingDtor → SoeUtil::Array<PendingNotificationImage,0,1>::Scala…` | ✅ | virtual | Installs the typed array vtable, clears the count at this+8, releases the data pointer at this+4 with alignment 4 and nulls it; when deleting-flag bit 0 is set, frees this, then returns this. |
