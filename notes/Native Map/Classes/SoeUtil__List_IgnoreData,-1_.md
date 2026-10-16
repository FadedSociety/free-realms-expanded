# Class: SoeUtil::List<IgnoreData,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c7e4b0` | `— ⚠reject` | ⚪ | function | Initializes the pool, requests a pooled block and invokes its virtual constructor when available; otherwise allocates 0x20 bytes aligned to 8. |
| `00c7e4f0` | `IgnoreData::Free` | ✅ | function | Passes the pointer and alignment 8 to FUN_00401c00 for release. |
| `00c80a10` | `IgnoreDataListDtor` | ✅ | dtor | Installs the IgnoreData list vtable and removes nodes from the head until the list is empty. |
| `00c80ba0` | `IgnoreDataListScalarDeletingDtor` | ✅ | dtor | Removes all IgnoreData list nodes, then frees the list object only when flag bit 0 is set. |
