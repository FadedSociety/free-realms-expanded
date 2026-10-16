# Class: SoeUtil::List<ChatNode,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0095b5d0` | `List::AllocateNode` | ✅ | virtual | Initializes allocator state, obtains the current allocator, and calls its virtual slot +4 if present; otherwise it tries the global allocator at DAT_01bdf280 through slot +4. Only when neither allocator is available doe… |
| `0095b610` | `ListFreeNode` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 8. |
| `0096f490` | `ChatNodeListDtor` | ✅ | dtor | Installs the ChatNode list vtable and repeatedly destroys the current head node until the list is empty. |
| `0096f500` | `ChatNodeListScalarDeletingDtor` | ✅ | dtor | Destroys all ChatNode list entries and frees object storage when the deleting flag's low bit is set. |
| `00b12770` | `ChatNodeListBaseDtor → ChatNodeList10Dtor` | ✅ | dtor | Drains list nodes, frees both child subtrees of the block tree, clears tree metadata, and then drains the base list; installs the inline-10 and unbounded-list vtables during teardown. |
