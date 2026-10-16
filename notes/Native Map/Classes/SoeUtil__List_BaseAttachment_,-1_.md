# Class: SoeUtil::List<BaseAttachment*,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0095b8f0` | `List::AllocateNode` | ✅ | virtual | Initializes allocator state, obtains the current allocator, and calls its virtual slot +4 if present; otherwise it tries the global allocator at DAT_01bdf280 through slot +4. Only when neither allocator is available doe… |
| `0095b930` | `ListFreeNode` | ✅ | virtual | Releases the supplied pointer through FUN_00401c00 with alignment 4. |
| `009643b0` | `BaseAttachmentListDestructor` | ✅ | dtor | Installs the List<BaseAttachment*> vtable and calls FUN_00962bc0 to clear its elements. |
| `009644c0` | `BaseAttachmentListScalarDeletingDestructor` | ✅ | dtor | Restores the typed list vtable, clears the list through FUN_00962bc0, and frees param_1 when flag bit 0 is set. |
