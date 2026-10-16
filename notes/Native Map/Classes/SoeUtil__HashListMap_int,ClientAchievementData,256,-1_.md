# Class: SoeUtil::HashListMap<int,ClientAchievementData,256,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ba9da0` | `HashListMap::Init → SoeUtil::HashListMap<int,ClientAchievementData,256,-1>::Has…` | ✅ | ctor | Installs the HashListMap vtable, clears three fields, zeroes 0x400 bytes of bucket storage, and returns this. |
| `00baa560` | `HashListMap::AllocateNode` | ✅ | method | Runs allocator setup, calls slot +4 on the returned allocator object or global allocator when present, and otherwise requests 0x198 bytes aligned to 8. |
| `00baa5a0` | `HashListMap::FreeNode` | ✅ | method | Passes the supplied pointer and alignment 8 to FUN_00401c00, then returns. |
| `00babb20` | `HashListMap::~HashListMap` | ✅ | dtor | Installs the map vtable and repeatedly destroys the current node at field +8 through FUN_0092ad20 until empty. |
| `00babdd0` | `HashListMap::~HashListMap → SoeUtil::HashListMap<int,ClientAchievementData,256,…` | ✅ | dtor | Installs the 256,8 map vtable, drains linked nodes, frees the auxiliary tree at index 0x105 with alignment 8, clears two fields, then installs the 256,-1 vtable and drains that chain too. |
