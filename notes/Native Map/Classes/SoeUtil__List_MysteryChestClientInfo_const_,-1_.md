# Class: SoeUtil::List<MysteryChestClientInfo_const*,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c7660` | `MysteryChestInfoListAlloc` | ✅ | function | If the current allocator manager exists, calls its allocation slot and exits; otherwise it tries the global manager and exits if that manager exists; the fallback request occurs only when both managers are absent (size … |
| `009c76a0` | `MysteryChestInfoListFree` | ✅ | function | Calls the shared free routine with alignment 4 on the supplied list node and returns. |
| `009c80d0` | `MysteryChestPointerListDtor` | ✅ | dtor | Writes the pointer-list vtable and calls its list clearing routine. |
| `009c80e0` | `MysteryChestPointerListScalarDeletingDtor` | ✅ | dtor | Writes the pointer-list vtable, clears its nodes, frees the allocation when the deleting flag is set, and otherwise returns the pointer. |
