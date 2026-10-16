# Class: SoeUtil::List<SaleDisplayInfo,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b24ad0` | `— ⚠reject` | ⚪ | virtual | Calls FUN_006fb9c0, then FUN_006fb860. If the thread-local manager is non-null, invokes its vtable slot +4 and returns; otherwise, if DAT_01bdf280 is non-null, invokes that manager slot +4 and returns; otherwise calls F… |
| `00b24b10` | `SoeUtil::List_SaleDisplayInfoFreeSlot` | ✅ | virtual | Virtual free slot (vtable index 3): FreeMemory(param_1, 4) on the node pointer. |
| `00b2aa90` | `SaleDisplayList::Dtor` | ✅ | dtor | Restores the SaleDisplayInfo list vtable and drains all remaining linked-list nodes. |
| `00b2aad0` | `SaleDisplayList::ScalarDeletingDtor` | ✅ | virtual | Restores the list vtable, drains its nodes, and conditionally frees the list object according to the deleting flag. |
