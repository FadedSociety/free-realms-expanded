# Class: SoeUtil::HashListMap<int,AssetManagement::Asset*,8192,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bd430` | `AssetPointerHashListMap8192::AllocateNode` | ✅ | virtual | Initializes allocator state, tries thread and global allocation, then allocates 0x14 bytes aligned to 4. |
| `008bd470` | `AssetPointerHashListMap8192::FreeAllocation` | ✅ | virtual | Calls FreeMemory with the supplied pointer and 4-byte alignment, then returns. |
| `008d3eb0` | `AssetHashListMap_Dtor ? ⚠downgrade` | 🟡 | dtor | Destructor of the asset hash-list map: rewrites its own vtable to the HashListMap vtable and then repeatedly frees the head node at param_1[2] through FUN_008bd3c0 until the bucket list is empty; it is reached only from… |
| `008d4270` | `AssetHashListMap_ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (slot [0] of the asset hash-list-map vtable): rewrites the object vtable to the HashListMap vtable, frees every bucket node at param_1[2] through FUN_008bd3c0, and then, when bit 0 of the… |
