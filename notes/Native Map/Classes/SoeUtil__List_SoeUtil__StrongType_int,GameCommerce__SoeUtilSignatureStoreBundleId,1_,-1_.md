# Class: SoeUtil::List<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleId,1>,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bd150` | `SignatureStoreBundleIdList::AllocateNode` | ✅ | virtual | Initializes allocator state, tries the thread and global allocators, then allocates 0xc bytes aligned to 4. |
| `008bd190` | `SignatureStoreBundleIdList::FreeNode` | ✅ | virtual | Calls FreeMemory with the supplied pointer and 4-byte alignment, then returns. |
| `008e0a30` | `ListStoreBundleId_Dtor` | ✅ | dtor | Stores the StoreBundleId list vtable and jumps to its list cleanup routine. |
| `008e2150` | `ListStoreBundleId_ScalarDeletingDtor` | ✅ | dtor | Installs the list vtable, runs list cleanup, and optionally frees the object when flag bit 0 is set. |
