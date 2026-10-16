# Class: SoeUtil::HashListMap<int,GameCommerce::StoreBundleCategoryGroupDefinition,64,16>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c3c330` | `SoeUtil::HashListMap::HashListMap` | ✅ | ctor | Constructor of the store-bundle-category-group map: stores the base (64,-1) vtable, zeroes this+8/+0xc/+0x10 and the 0x100-byte bucket array at this+0x14 (64 buckets), stores the derived (64,16) vtable and constructs th… |
| `00c3cc80` | `— ⚠reject` | ⚪ | method | Allocates one 0x24-byte slot for the 64/16 pool: grows via 00c3cb00(0) when the free list at this+8 is empty, pops the head, decrements this+0xc and, when the auto-shrink flag at this+0x10 is set and the slot lies in it… |
| `00c3ccf0` | `— ⚠reject` | ⚪ | method | Returns a slot to the 64/16 pool: pushes it onto the free list at this+8, increments this+0xc and, when the auto-shrink flag at this+0x10 is set, finds the owning block (00c3c5d0) and bumps its used counter at +0x18 - r… |
| `00c3cd60` | `PoolAllocSlotThunk` | ✅ | virtual | Adjustor thunk in vtable slot 1: adds 0x114 to ECX and jumps to 00c3cc80, forwarding the map's virtual slot-allocation call to the pool sub-object at +0x114. |
| `00c3cd70` | `PoolFreeSlotThunk` | ✅ | virtual | Adjustor thunk in vtable slot 2: adds 0x114 to ECX and jumps to 00c3ccf0, forwarding the map's virtual slot-release call to the pool sub-object at +0x114. |
| `00c3cd80` | `SoeUtil::HashListMap::~HashListMap` | ✅ | dtor | Deleting destructor of the (64,16) category-group map: stores its vtable, frees the free-list at this+8 via 00b28dd0, releases the pool block-tree root at this+0x114 (children via 00c37a80, memory via FreeMemory 00401c0… |
| `00c3db50` | `SoeUtil::HashListMap::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of the derived (64,16) category-group map: runs the destructor 00c3cd80 and, when bit 0 of the flag byte is set, releases the object with _free; returns the object pointer. |
