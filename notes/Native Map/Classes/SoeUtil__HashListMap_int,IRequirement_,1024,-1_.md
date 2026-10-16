# Class: SoeUtil::HashListMap<int,IRequirement*,1024,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bb9c30` | `HashListMap::InitIntRequirementMap1024` | ✅ | ctor | Installs the hash-list-map vtable, zeros map counters and state, clears 0x1000 bytes of buckets, and returns the receiver. |
| `00bbafc0` | `SoeUtil::HashListMap::AllocateNode` | ✅ | method | Tries the current thread allocator and global allocator for a block; if neither is available, allocates 0x14 bytes aligned to 4 and returns it. |
| `00bbb000` | `SoeUtil::HashListMap::FreeNode` | ✅ | method | Passes the supplied block to the memory manager free routine with alignment 4. |
