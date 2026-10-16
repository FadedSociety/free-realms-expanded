# Class: SoeUtil::HashMap<IRequirement*,SoeGems::ObjectFactory<IRequirement,int>::BasePool*,32768,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bb9e90` | `HashMap::InitRequirementPoolMap32768` | ✅ | ctor | Installs the hash-map vtable, zeros map counters, clears 0x20000 bytes of buckets, and returns the receiver. |
| `00bbb080` | `SoeUtil::HashMap::~HashMap` | ✅ | dtor | Restores the hash-map vtable, then clears its 32768-bucket storage through 00bb9ca0. |
| `00bbb090` | `SoeUtil::HashMap::AllocateNode` | ✅ | method | Tries the current thread allocator and global allocator for a block; if neither is available, allocates 0xc bytes aligned to 4 and returns it. |
| `00bbb0d0` | `SoeUtil::HashMap::FreeNode` | ✅ | method | Passes the supplied block to the memory manager free routine with alignment 4. |
| `00bbb330` | `SoeUtil::HashMap::~HashMap` | ✅ | dtor | Restores the map vtable, clears buckets through 00bb9ca0, and calls _free(this) only when flag bit 0 is set. |
