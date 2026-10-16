# Class: SoeUtil::List<BattleMagesProxiedProjectile,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b70250` | `SoeUtil::List::AllocateNode` | ✅ | method | Node allocator (vtable slot 2) of the proxied-projectile list: asks FUN_006fb860 for a block, falls back to the global allocator at DAT_01bdf280, and finally calls FUN_006fb950(0x90,0x10) to allocate a 144-byte node wit… |
| `00b70290` | `SoeUtil::List::DeallocateNode` | ✅ | method | Node deallocator (vtable slot 3) of the proxied-projectile list: forwards its argument to the memory manager with alignment 0x10 (FUN_00401c00(param_1,0x10)). |
| `00b70d90` | `SoeUtil::List::~List` | ✅ | dtor | Non-deleting destructor of the proxied-projectile list: rewrites +0 with SoeUtil::List<BattleMagesProxiedProjectile,-1>::vftable and then repeatedly calls 00b707e0 on the head stored at +4 until the list is empty; it ne… |
| `00b70e00` | `SoeUtil::List::ScalarDeletingDestructor` | ✅ | dtor | Virtual deleting destructor at the proxied-projectile list's vtable slot 0: writes SoeUtil::List<BattleMagesProxiedProjectile,-1>::vftable over +0, drains the list by calling 00b707e0 on the head at +4 until empty, then… |
