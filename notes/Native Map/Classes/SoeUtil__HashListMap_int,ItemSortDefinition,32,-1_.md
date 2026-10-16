# Class: SoeUtil::HashListMap<int,ItemSortDefinition,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01026970` | `HashListMapCtor32` | ✅ | ctor | Constructor: installs the HashListMap<int,ItemSortDefinition,32,-1> vtable, clears +0x10, +0x8, +0xc, memsets 0x80 bytes at +0x14 (32 buckets), clears +4, and returns this. |
| `01026b90` | `HashListMapItemSortAllocSlot` | ✅ | virtual | Virtual allocate slot: calls the lock helper 006fb9c0, then allocates through the thread-manager entry 006fb860 and its vslot +4, else the global allocator DAT_01bdf280 vslot +4, else 006fb950 with size 0x1c and alignme… |
| `01026bd0` | `HashListMapItemSortFreeSlot` | ✅ | virtual | Virtual free slot: bytes 8b 44 24 04 read the pointer, push 4, push pointer, call 00401c00 (FreeMemory with alignment 4), then ret 4. |
| `010271b0` | `HashListMapItemSortDtorBody` | ✅ | dtor | Destructor body: stores the HashListMap<int,ItemSortDefinition,32,-1> vtable at offset 0, then removes list heads through 01026e80 until the list is empty. |
