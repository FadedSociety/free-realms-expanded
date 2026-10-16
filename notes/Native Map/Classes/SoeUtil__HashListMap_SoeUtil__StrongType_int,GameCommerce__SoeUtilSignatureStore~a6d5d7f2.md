# Class: SoeUtil::HashListMap<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleGroupId,1>,GameCommerce::StoreBundleGroupDefinition,16,4>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c3b820` | `SoeUtil::HashListMap::HashListMap` | ✅ | ctor | Constructor of the bundle-group definition map: stores the base (16,-1) vtable, zeroes this+8/+0xc/+0x10 and the 0x40-byte bucket array at this+0x14 (16 buckets), stores the derived (16,4) vtable, zeroes the pool fields… |
| `00c3c190` | `PoolAllocSlot` | 🟢 | method | Allocates one 0x80-byte slot for the bundle-group map: grows the pool through 00c3c010(0) when the free list at this+8 is empty, pops the head, decrements this+0xc and, when the auto-shrink flag at this+0x10 is set and … |
| `00c3c1e0` | `PoolFreeSlot` | 🟢 | method | Returns a slot to the bundle-group pool: pushes it onto the free list at this+8, increments this+0xc and, when the auto-shrink flag at this+0x10 is set, finds the owning block (00c3baf0) and bumps its used counter at +0… |
| `00c3c240` | `PoolAllocSlotThunk` | ✅ | virtual | Adjustor thunk in vtable slot 1: adds 0x54 to ECX and jumps to 00c3c190, forwarding the map's virtual slot-allocation call to the pool sub-object at +0x54. |
| `00c3c250` | `PoolFreeSlotThunk` | ✅ | virtual | Adjustor thunk in vtable slot 2: adds 0x54 to ECX and jumps to 00c3c1e0, forwarding the map's virtual slot-release call to the pool sub-object at +0x54. |
| `00c3c260` | `— ⚠reject` | ⚪ | dtor | Deleting destructor of the derived (16,4) bundle-group map: stores its vtable, frees the free-list at this+8 via 00b28d20, releases the pool block-tree root at this+0x54 (children via 00c37980, memory via FreeMemory 004… |
| `00c3db30` | `SoeUtil::HashListMap::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of the derived (16,4) bundle-group map: runs the destructor 00c3c260 and, when bit 0 of the flag byte is set, releases the object with _free; returns the object pointer. |
