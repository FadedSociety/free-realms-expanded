# Class: SoeUtil::HashMap<int,SoeGems::ObjectFactory<IRequirement,int>::BasePool*,1024,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bb9f20` | `HashMap::InitRequirementPoolIdMap1024` | ✅ | ctor | Installs the hash-map vtable, zeros map counters, clears 0x1000 bytes of buckets, and returns the receiver. |
| `00bbb160` | `SoeUtil::HashMap::~HashMap` | ✅ | dtor | Restores the hash-map vtable, then clears the map through 00bb9d60. |
| `00bbb170` | `SoeUtil::HashMap::AllocateNode` | ✅ | method | Tries the current thread allocator and global allocator for a block; if neither is available, allocates 0xc bytes aligned to 4 and returns it. |
| `00bbb1b0` | `SoeUtil::HashMap::FreeNode` | ✅ | method | Passes the supplied block to the memory manager free routine with alignment 4. |
| `00bbb390` | `SoeUtil::HashMap::~HashMap` | ✅ | dtor | Restores the map vtable, clears buckets through 00bb9d60, and calls _free(this) only when flag bit 0 is set. |
| `00bbb410` | `SoeUtil::HashMap::~HashMap` | ✅ | dtor | Restores the map vtable and invokes its bucket-clear routine inside an SEH cleanup frame. |
| `00bbc160` | `SoeUtil::HashMap::InitPoolMap1024 → SoeUtil::Hash<SoeGems::ObjectFactory<IRequi…` | ✅ | ctor | Installs the integer-to-pool map vtable, zeros its counters, clears 0x1000 bytes of bucket storage, and returns the receiver. |
| `00bbc240` | `SoeUtil::HashMap::~HashMap → SoeUtil::Hash<SoeGems::ObjectFactory<IRequirement,…` | ✅ | dtor | Restores the map vtable, clears buckets through 00bb9d60, frees this object only when flag bit 0 is set, and restores the SEH frame. |
