# Class: SoeUtil::List<ProxiedCharacterAttachmentGroup*,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bf1080` | `SoeUtil::ListAttachmentGroup::AllocateNode` | ✅ | virtual | Initializes a list allocation helper, requests a node-sized block from the current allocator when available, otherwise uses the global allocator, then returns the block. |
| `00bf10c0` | `SoeUtil::ListAttachmentGroup::FreeNode` | ✅ | virtual | Frees the supplied list node using the allocator with alignment 4. |
| `00bf13e0` | `SoeUtil::ListAttachmentGroup::DtorThunk → SoeUtil::ListAttachmentGroup::Dtor` | ✅ | dtor | Jumps directly to 00bf1380, the list clear routine, after installing no additional teardown logic. |
| `00bf13f0` | `SoeUtil::ListAttachmentGroup::ScalarDeletingDtor` | ✅ | dtor | Installs the list vtable, clears all linked nodes through FUN_00bf1380, frees the object when flag bit 0 is set, and otherwise returns its pointer. |
