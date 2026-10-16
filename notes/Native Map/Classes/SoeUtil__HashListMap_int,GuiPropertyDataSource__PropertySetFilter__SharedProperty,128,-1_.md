# Class: SoeUtil::HashListMap<int,GuiPropertyDataSource::PropertySetFilter::SharedProperty,128,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bde20` | `SharedPropertyHashListMap128::Allocate` | ✅ | virtual | Allocates a 0x1c-byte map entry aligned to 4 after trying the thread and global allocators. |
| `008bde60` | `SharedPropertyHashListMap128::FreeAllocation` | ✅ | virtual | Releases the supplied map entry with 4-byte alignment. |
| `008e0e50` | `HashListMapSharedProperty_Dtor` | ✅ | dtor | Installs the shared-property map vtable and repeatedly destroys its head node until empty. |
| `008e2250` | `HashListMapSharedProperty_ScalarDeletingDtor` | ✅ | dtor | Installs the map vtable, destroys linked nodes, and optionally frees the object when flag bit 0 is set. |
| `008ebc30` | `GuiSharedPropertyHashListMapDtor` | ✅ | dtor | Installs the exact SharedProperty map vtable, then while the head at +8 is nonnull calls 0085c530 with the map in ECX and the current head pushed as its argument. |
| `008f4310` | `HashListMapSharedProperty_Ctor` | ✅ | ctor | Installs the SharedProperty map vtable, clears its map header, zeroes 0x200 bytes of inline storage, and installs the HashList base vtable. |
