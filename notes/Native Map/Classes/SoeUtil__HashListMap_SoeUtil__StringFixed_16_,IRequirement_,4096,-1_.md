# Class: SoeUtil::HashListMap<SoeUtil::StringFixed<16>,IRequirement*,4096,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bb9c70` | `HashListMap::InitStringRequirementMap4096` | ✅ | ctor | Installs the hash-list-map vtable, zeros map counters and state, clears 0x4000 bytes of buckets, and returns the receiver. |
| `00bbb020` | `SoeUtil::HashListMap::AllocateNode` | ✅ | method | Tries the current thread allocator and global allocator for a block; if neither is available, allocates 0x34 bytes aligned to 4 and returns it. |
| `00bbb060` | `SoeUtil::HashListMap::FreeNode` | ✅ | method | Passes the supplied block to the memory manager free routine with alignment 4. |
