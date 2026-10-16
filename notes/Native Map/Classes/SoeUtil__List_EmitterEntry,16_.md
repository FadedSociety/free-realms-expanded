# Class: SoeUtil::List<EmitterEntry,16>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b09790` | `PopFreeNodeAdjustor` | 🟢 | virtual | Adds 0x10 to ECX, then jumps to 00b096b0. |
| `00b097a0` | `PushFreeNodeAdjustor` | 🟢 | virtual | Adds 0x10 to ECX, then jumps to 00b09720. |
| `00b097b0` | `EmitterEntryListDestructor` | 🟢 | dtor | Sets inline-list vtable, removes entries, frees tree/pool storage and restores the base list vtable. |
| `00b09880` | `ScalarDeletingDtor ? → SoeUtil::List<EmitterEntry,16>::ScalarDeletingDtor` | 🟡 | dtor | Destroys the EmitterEntry list and optionally frees its containing object. |
