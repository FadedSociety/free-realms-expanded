# Class: ClientQuickChatManager::InternalEmoteItemCategoriesDs

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b8afc0` | `QuickChatCategoryDs::ColumnName` | ✅ | method | Returns Id for index zero, CategoryName for index one, and the error marker for other indices. |
| `00b8c400` | `InternalEmoteItemCategoriesDs::GetNestedFieldFC0` | ✅ | virtual | Returns the dword at nested object offset 0xfc0 reached through this object's field at offset 0x224. |
| `00b935c0` | `ClientQuickChatManager::InternalEmoteItemCategoriesDs::GetField` | ✅ | virtual | Finds the category row by key, then formats its numeric field or resolves the string identifier through the client string table; invalid field IDs emit the error string and return false. |
| `00b960f0` | `ClientQuickChatManager_InternalEmoteItemCategoriesDs_Destruct` | ✅ | dtor | Destructor body of the emote item categories data source: stores the InternalEmoteItemCategoriesDs vtable into the three interface slots at +0, +0x30 and +0x34 and calls the base GuiUserDataTable destructor FUN_0091bf60… |
| `00b96ca0` | `ClientQuickChatManager_InternalEmoteItemCategoriesDs_Construct` | ✅ | ctor | Constructor of the emote item categories data source: runs the base IDataAccessTable/GuiUserDataTable constructor FUN_00922890 with param_2, stores the InternalEmoteItemCategoriesDs vtable into the three interface slots… |
| `00b96d20` | `ClientQuickChatManager_InternalEmoteItemCategoriesDs_DeletingDestruct` | ✅ | dtor | Scalar deleting destructor: calls the data source destructor 00b960f0 and then _free(this) when bit 0 of param_2 is set. |
