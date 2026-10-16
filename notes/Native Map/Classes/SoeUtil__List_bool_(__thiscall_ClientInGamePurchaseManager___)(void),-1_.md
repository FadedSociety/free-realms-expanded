# Class: SoeUtil::List<bool_(__thiscall_ClientInGamePurchaseManager::*)(void),-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b24e30` | `— ⚠reject` | ⚪ | virtual | Calls FUN_006fb9c0, then FUN_006fb860. If the thread-local manager is non-null, invokes its vtable slot +4 and returns; otherwise, if DAT_01bdf280 is non-null, invokes that manager slot +4 and returns; otherwise calls F… |
| `00b24e70` | `SoeUtil::List_ManagerBoolPredicateFreeSlot` | ✅ | virtual | Virtual free slot (vtable index 3): FreeMemory(param_1, 4) on the node pointer. |
| `00b2aaa0` | `PurchaseCallbackList::Dtor` | ✅ | dtor | Restores the callback-list vtable and drains the linked list through its specialized removal helper. |
| `00b2ab00` | `PurchaseCallbackList::ScalarDeletingDtor` | ✅ | virtual | Restores the callback-list vtable, drains nodes, and conditionally frees the list object. |
