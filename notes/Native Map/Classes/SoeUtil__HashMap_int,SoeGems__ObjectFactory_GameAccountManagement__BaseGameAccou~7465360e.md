# Class: SoeUtil::HashMap<int,SoeGems::ObjectFactory<GameAccountManagement::BaseGameAccountPacket,int>::BasePool*,1024,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `007069f0` | `BasePoolMapConstructor ?` | 🟡 | ctor | Initializes the integer-keyed base-pool map and clears its 1024-bucket table. |
| `00708cb0` | `HashMapDtor → ObjectFactoryBasePoolMapDtor` | ✅ | dtor | Stores the HashMap vtable, then tail-jumps to 00706a20 for table cleanup; this function has no return after the jump. |
| `00708cc0` | `HashMapAlloc → ObjectFactoryBasePoolHashNodeAlloc` | ✅ | virtual | Uses the current memory manager or global fallback to allocate 12 bytes aligned to 4. |
| `00708d00` | `HashMapFree` | ✅ | virtual | Passes the supplied pointer and alignment 4 to the memory-manager free routine. |
| `00709180` | `ScalarDeleteHashMap` | ✅ | dtor | Stores this map's vtable, calls 00706a20 to destroy its table storage, and frees the object when the delete flag is set. |
