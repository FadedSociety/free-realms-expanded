# Class: SoeUtil::HashMap<int,SoeGems::ObjectFactory<GameAccountManagement::BaseGameAccountPacket,int>::ICreator*,1024,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00706960` | `CreatorMapConstructor ?` | 🟡 | ctor | Initializes the integer-keyed creator map and clears its 1024-bucket table. |
| `00708c40` | `HashMapDtor` | ✅ | dtor | Stores the HashMap vtable, then tail-jumps to 00706990 for table cleanup; this function has no return after the jump. |
| `00708c50` | `HashMapAlloc` | ✅ | virtual | Uses the current memory manager or global fallback to allocate 12 bytes aligned to 4. |
| `00708c90` | `HashMapFree` | ✅ | virtual | Passes the supplied pointer and alignment 4 to the memory-manager free routine. |
| `00709150` | `ScalarDeleteHashMap` | ✅ | dtor | Stores this map's vtable, calls 00706990 to destroy its table storage, and frees the object when the delete flag is set. |
