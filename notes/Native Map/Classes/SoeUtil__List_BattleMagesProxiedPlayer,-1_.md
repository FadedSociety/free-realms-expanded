# Class: SoeUtil::List<BattleMagesProxiedPlayer,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b701e0` | `SoeUtil::List::AllocateNode → SoeUtil::List<BattleMagesProxiedPlayer,-1>::Alloc…` | ✅ | method | Node allocator (vtable slot 2) of the proxied-player list: asks the per-thread allocator helper FUN_006fb860 for a block, and when that returns NULL falls back to the global at DAT_01bdf280, finally calling FUN_006fb950… |
| `00b70220` | `SoeUtil::List::DeallocateNode → SoeUtil::List<BattleMagesProxiedPlayer,-1>::Dea…` | ✅ | method | Node deallocator (vtable slot 3) of the proxied-player list: forwards its argument to the memory manager with alignment 8 (FUN_00401c00(param_1,8)) and returns nothing else. |
| `00b70d60` | `SoeUtil::List::~List` | ✅ | dtor | Non-deleting destructor of the proxied-player list: rewrites +0 with SoeUtil::List<BattleMagesProxiedPlayer,-1>::vftable and then repeatedly calls 00b70750 on the head stored at +4 until the list is empty; it never free… |
| `00b70dc0` | `SoeUtil::List::ScalarDeletingDestructor` | ✅ | dtor | Virtual deleting destructor at the proxied-player list's vtable slot 0: writes SoeUtil::List<BattleMagesProxiedPlayer,-1>::vftable over +0, drains the list by calling 00b70750 on the head at +4 until empty, then calls _… |
