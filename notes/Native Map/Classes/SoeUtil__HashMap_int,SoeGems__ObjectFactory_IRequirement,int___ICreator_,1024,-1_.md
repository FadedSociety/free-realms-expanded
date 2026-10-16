# Class: SoeUtil::HashMap<int,SoeGems::ObjectFactory<IRequirement,int>::ICreator*,1024,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bb9ec0` | `HashMap::InitRequirementCreatorMap1024` | ✅ | ctor | Installs the hash-map vtable, zeros map counters, clears 0x1000 bytes of buckets, and returns the receiver. |
| `00bbb0f0` | `SoeUtil::HashMap::~HashMap` | ✅ | dtor | Restores the hash-map vtable, then clears the map through 00bb9d00. |
| `00bbb100` | `SoeUtil::HashMap::AllocateNode` | ✅ | method | Tries the current thread allocator and global allocator for a block; if neither is available, allocates 0xc bytes aligned to 4 and returns it. |
| `00bbb140` | `SoeUtil::HashMap::FreeNode` | ✅ | method | Passes the supplied block to the memory manager free routine with alignment 4. |
| `00bbb360` | `SoeUtil::HashMap::~HashMap` | ✅ | dtor | Restores the map vtable, clears buckets through 00bb9d00, and calls _free(this) only when flag bit 0 is set. |
| `00bbb3c0` | `SoeUtil::HashMap::~HashMap` | ✅ | dtor | Restores the map vtable and invokes its bucket-clear routine inside an SEH cleanup frame. |
| `00bbc0e0` | `SoeUtil::HashMap::InitCreatorMap1024 → SoeUtil::Hash<SoeGems::ObjectFactory<IRe…` | ✅ | ctor | Installs the integer-to-creator map vtable, zeros its counters, clears 0x1000 bytes of bucket storage, and returns the receiver. |
| `00bbc1e0` | `SoeUtil::HashMap::~HashMap → SoeUtil::Hash<SoeGems::ObjectFactory<IRequirement,…` | ✅ | dtor | Restores the map vtable, clears buckets through 00bb9d00, frees this object only when flag bit 0 is set, and restores the SEH frame. |
