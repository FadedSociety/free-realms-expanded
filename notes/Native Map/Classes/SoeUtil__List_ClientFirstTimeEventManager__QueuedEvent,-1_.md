# Class: SoeUtil::List<ClientFirstTimeEventManager::QueuedEvent,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a1cf70` | `FirstTimeEventQueue_Dtor` | 🟢 | dtor | Installs the queued-event list vftable and destroys nodes through 00a1b860 until the list is empty. |
| `00a1cfb0` | `FirstTimeEventQueue_AllocNode` | 🟢 | virtual | Requests aligned 0xcb8-byte memory from the current or global allocator and returns the node storage. |
| `00a1cff0` | `FirstTimeEventQueue_FreeNode` | 🟢 | virtual | Calls aligned allocator free 00401c00 for the supplied pointer with alignment four, then returns. |
| `00a1d010` | `FirstTimeEventQueue_ScalarDeletingDtor` | 🟢 | dtor | Destroys all queued event nodes through 00a1b860, optionally frees the list object, and otherwise returns it. |
