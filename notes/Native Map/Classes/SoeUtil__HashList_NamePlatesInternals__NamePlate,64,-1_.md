# Class: SoeUtil::HashList<NamePlatesInternals::NamePlate,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009d11e0` | `NamePlatesInternals::NamePlateHashList::NodeAlloc` | ✅ | virtual | Runs allocator setup, obtains the current allocator or fallback allocator, and requests a 0x34-byte block aligned to four bytes. |
| `009d1220` | `NamePlatesInternals::NamePlateHashList::NodeFree` | ✅ | virtual | Frees the supplied node pointer through FUN_00401c00 with four-byte alignment. |
| `009d1bc0` | `NamePlatesInternals::NamePlateHashList::ScalarDeletingDtor → SoeUtil::HashList<…` | ✅ | dtor | Runs FUN_009d1a90 to release entries, then frees the list object when the deleting flag is set. |
