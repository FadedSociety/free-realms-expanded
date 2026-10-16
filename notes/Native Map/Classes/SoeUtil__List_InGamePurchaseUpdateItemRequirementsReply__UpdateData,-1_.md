# Class: SoeUtil::List<InGamePurchaseUpdateItemRequirementsReply::UpdateData,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b24b40` | `— ⚠reject` | ⚪ | virtual | Calls FUN_006fb9c0, then FUN_006fb860. If the thread-local manager is non-null, invokes its vtable slot +4 and returns; otherwise, if DAT_01bdf280 is non-null, invokes that manager slot +4 and returns; otherwise calls F… |
| `00b24b80` | `SoeUtil::List_UpdateItemRequirementsDataFreeSlot` | ✅ | virtual | Virtual free slot (vtable index 3): FreeMemory(param_1, 4) on the node pointer. |
| `00b2c810` | `UpdateItemRequirementsReplyDataList::Dtor` | ✅ | dtor | Restores the update-data list vtable and drains its nodes. |
| `00b2cba0` | `UpdateItemRequirementsReplyDataList::ScalarDeletingDtor` | ✅ | virtual | Restores the list vtable, drains all records, and conditionally frees the list object. |
