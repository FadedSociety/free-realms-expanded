# Class: ClientQuickChatManager::InternalDs

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b8af00` | `QuickChatInternalDs::ColumnName` | ✅ | method | Maps column indices 0 through 11 to Id, ParentId, MenuText, ChatText, AnimationId, MenuIconId, ItemId, ItemCategory, ItemCount, IsFavorite, IsRecentlyUsed, and RecentIndex. |
| `00b8c390` | `— ⚠reject` | ⚪ | virtual | Calls a data-source virtual method, iterates rows in the nested collection, conditionally invokes row cleanup for entries with valid IDs, then calls another virtual method. |
| `00b933f0` | `ClientQuickChatManager::InternalDs::GetField` | ✅ | virtual | Checks row index against the source count, fetches the row and indexed entry, then switches over field IDs to return or format values; invalid IDs emit the error string and return false. |
| `00b96070` | `ClientQuickChatManager_InternalDs_Destruct` | ✅ | dtor | Destructor body of the quick-chat data source: stores the InternalDs vtable into the three interface slots at +0, +0x30 and +0x34 and calls the base GuiUserDataTable destructor FUN_0091bf60 inside an SEH frame. |
| `00b96c00` | `ClientQuickChatManager_InternalDs_Construct ⚠downgrade` | 🟢 | ctor | Constructor of the quick-chat data source: runs the base IDataAccessTable/GuiUserDataTable constructor FUN_00922890 with param_2, then stores the InternalDs vtable into the three interface slots at +0, +0x30 and +0x34 a… |
| `00b96c80` | `ClientQuickChatManager_InternalDs_DeletingDestruct` | ✅ | dtor | Scalar deleting destructor: calls the data source destructor 00b96070 and then _free(this) when bit 0 of param_2 is set. |
