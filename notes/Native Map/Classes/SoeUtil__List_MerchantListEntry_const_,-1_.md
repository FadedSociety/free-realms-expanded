# Class: SoeUtil::List<MerchantListEntry_const*,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cfe0b0` | `ArrayListAllocateNode → ListAllocateNode` | ✅ | virtual | Routes allocation of a 12-byte list node at alignment four through the current manager, then the fallback manager, then the CRT allocation helper. |
| `00cfe0f0` | `ArrayListFreeNode → ListFreeNode` | ✅ | virtual | Passes the supplied list-node pointer to the shared allocator free path at alignment four. |
| `00cfe830` | `ListCleanup` | ✅ | dtor | Restores the list vtable and clears the list by repeatedly unlinking and cleaning each remaining node. |
| `00cfe840` | `ListScalarDeletingDtor` | ✅ | dtor | Restores the list vtable, clears its remaining nodes, and frees the list object only when deleting-flag bit 0 is set. |
