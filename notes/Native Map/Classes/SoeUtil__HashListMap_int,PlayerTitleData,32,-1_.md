# Class: SoeUtil::HashListMap<int,PlayerTitleData,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b498e0` | `SoeUtil::HashListMap::Ctor` | ✅ | ctor | Constructs the hash map: stores SoeUtil::HashListMap<int,PlayerTitleData,32,-1>::vftable (01821284), zeroes +4, +8, +0xc and +0x10, and memsets 0x80 bytes at +0x14, i.e. the 32 bucket heads of the map (32 buckets x 4 by… |
| `00b499e0` | `SoeUtil::HashList::Find` | 🟢 | method | Key lookup: bucket = key & 0x1f, start at the bucket head at [this+0x18+bucket*4], then walk the chain through the +0x1c next pointer comparing the node key at +0x18 with the key; returns the node or 0. |
| `00b49a60` | `SoeUtil::HashList::AllocNode` | ✅ | virtual | Node allocator: asks the current allocator through FUN_006fb860 (with FUN_006fb9c0 first); if one is returned it calls its virtual slot 1 with size 0x20 and alignment 4, otherwise it falls back to the global allocator a… |
| `00b49aa0` | `SoeUtil::HashList::FreeNode → SoeUtil::HashList<PlayerTitleData,32,-1>::FreeNode` | ✅ | virtual | Node deallocator: pushes alignment 4 on the stack next to its pointer argument and calls FUN_00401c00 (the accepted FreeMemory helper, which routes to the thread memory manager); the raw bytes are mov eax,[esp+4]; push … |
| `00b49b30` | `SoeUtil::HashList::EraseKey` | 🟢 | method | Erase by key: bucket = key & 0x1f, walk the chain at [this+0x18+bucket*4] via +0x1c comparing the node key at +0x18 with the key; when found it calls FUN_008bff10(node) and then FUN_00cc55d0 (which in this class family … |
| `00b49bb0` | `SoeUtil::HashListMap::Dtor → SoeUtil::HashListMap<int,PlayerTitleData,32,-1>::D…` | ✅ | dtor | Installs SoeUtil::HashListMap<int,PlayerTitleData,32,-1>::vftable, then repeatedly calls 008bff10 on the node at this+8 until the chain is empty. |
| `00b49be0` | `SoeUtil::HashListMap::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores the vtable, frees the node chain at [this+8] through FUN_008bff10, and calls _free(this) when the low bit of the deleting flag is set, otherwise returns this. |
| `00b49c60` | `SoeUtil::HashList::ClearNodes → SoeUtil::HashListMap<int,PlayerTitleData,32,-1>…` | ✅ | method | Stores SoeUtil::HashListMap::vftable and then repeatedly frees the node chain at [this+8] through FUN_008bff10 until the head pointer is null, under an SEH frame. |
