# Class: SoeUtil::List<ShipCombatCannonConfig,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a61370` | `CannonConfigListDtor` | ✅ | dtor | Resets the list vtable and repeatedly removes the head node through 00a5ae80 until the list is empty. |
| `00a613b0` | `ListAllocateNode` | ✅ | virtual | Attempts allocation through the current allocator object's virtual slot +4; if absent, uses the global allocator object's slot +4, otherwise requests 0x80 bytes aligned to 0x10. |
| `00a613f0` | `ListFreeNode` | ✅ | virtual | Frees the supplied node with alignment 0x10 using the current memory manager helper. |
| `00a61660` | `CannonConfigList::ScalarDeletingDtor` | ✅ | dtor | Resets the list vtable, removes every node through 00a5ae80, then frees the list object only when delete flag bit 0 is set. |
| `00a616e0` | `CannonConfigListAssign` | 🟢 | method | When source and destination differ, drains all destination nodes, then walks the source through its +0x70 next link and appends a copied node for every source entry. |
