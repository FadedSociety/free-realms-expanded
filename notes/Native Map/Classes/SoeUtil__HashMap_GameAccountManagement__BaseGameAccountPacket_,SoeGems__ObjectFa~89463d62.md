# Class: SoeUtil::HashMap<GameAccountManagement::BaseGameAccountPacket*,SoeGems::ObjectFactory<GameAccountManagement::BaseGameAccountPacket,int>::BasePool*,32768,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `007068d0` | `ObjectFactoryAccountPacketMapConstructor ?` | 🟡 | ctor | Initializes the map count and clears its 32768-bucket pointer table. |
| `00708bd0` | `HashMapDtor → PacketPoolMapDtor` | ✅ | dtor | Stores this HashMap vtable, then tail-jumps to 00706900 for table cleanup; this function has no return after the jump. |
| `00708be0` | `HashMapAlloc → PacketPoolMapAlloc` | ✅ | virtual | Uses the current memory manager or global fallback to allocate 12 bytes aligned to 4. |
| `00708c20` | `HashMapFree` | ✅ | virtual | Passes the supplied pointer and alignment 4 to the memory-manager free routine. |
| `00709120` | `ScalarDeleteHashMap` | ✅ | dtor | Stores this map's vtable, calls 00706900 to destroy its table storage, and frees the object when the delete flag is set. |
