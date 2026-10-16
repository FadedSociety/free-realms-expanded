# Class: SoeUtil::List<ClientCollection_const*,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c321e0` | `ListAllocator` | ✅ | method | Initializes allocator state, tries current and global memory managers, and falls back to allocating 0xc bytes aligned to four. |
| `00c32220` | `ListFreeBuffer` | ✅ | method | Frees its supplied buffer by calling FUN_00401c00 at alignment 4. |
| `00c32450` | `— ⚠reject` | ⚪ | dtor | Restores the list vtable and clears all list elements through FUN_00c323f0. |
| `00c32460` | `ClientCollectionListScalarDeletingDtor` | ✅ | dtor | Restores the list vtable, clears elements, and frees the list object when flag bit 0 is set. |
