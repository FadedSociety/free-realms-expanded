# Class: SoeUtil::List<CoinStoreItemRecord,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c00620` | `CoinStoreItemList::AllocateNode → SoeUtil::List<CoinStoreItemRecord,-1>::Alloca…` | ✅ | virtual | Calls allocator helpers, asks a non-null allocator object through vtable offset 4 for memory, or falls back to allocating 0x20 bytes aligned to 4. |
| `00c00660` | `FreeListNodeWithAlignment4` | ✅ | virtual | Calls 00401c00 with the pointer argument and alignment 4, then returns. |
| `00c00c60` | `CoinStoreItemList::DtorBody` | ✅ | dtor | Restores the list vtable and repeatedly removes the current head node through 00c00840 until the list is empty. |
| `00c00c90` | `CoinStoreItemList::ScalarDeletingDtor → SoeUtil::List<CoinStoreItemRecord,-1>::…` | ✅ | dtor | Restores the list vtable, removes all nodes through 00c00840, then frees this only when flag bit 0 is set. |
| `00c00cd0` | `CoinStoreItemList::Ctor → SoeUtil::List<CoinStoreItemRecord,-1>::Ctor` | ✅ | ctor | Initializes four endpoints from sentinel globals, installs the list vtable at offset 16, and clears three node/count fields. |
| `00c00d50` | `CoinStoreItemList::DtorBody → SoeUtil::List<CoinStoreItemRecord,-1>::DtorBody` | ✅ | dtor | Restores the embedded list vtable at offset 16 and repeatedly removes its head node at offset 20 through 00c00840 until empty. |
