# Class: SoeUtil::List<NamePlatesInternals::DebugNode,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009d1240` | `NamePlatesInternals::DebugNode::NodeAlloc` | ✅ | virtual | Runs allocator setup, tries the current allocator and fallback allocator, then requests a 0x38-byte block aligned to four bytes. |
| `009d1280` | `NamePlatesInternals::DebugNode::NodeFree` | ✅ | virtual | Frees the supplied list-node pointer through FUN_00401c00 with four-byte alignment. |
| `009d1be0` | `NamePlatesInternals::DebugNodeList::Dtor` | ✅ | dtor | Installs the list vtable and repeatedly removes its head node at this+4 through FUN_009d17e0. |
| `009d1ce0` | `NamePlatesInternals::DebugNodeList::ScalarDeletingDtor` | ✅ | dtor | Installs the list vtable, drains all nodes through FUN_009d17e0, and frees the object when the deleting flag is set. |
