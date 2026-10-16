# Class: SoeUtil::HashListMap<int,GameCommerce::StorePortalCategoryDefinition,64,16>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c3a050` | `SoeUtil::HashListMap::HashListMap` | ✅ | ctor | Constructor of the portal-category definition map: stores the base (64,-1) vtable, zeroes this+8/+0xc/+0x10 and the 0x100-byte bucket-head array at this+0x14 (64 buckets), stores the derived (64,16) vtable and construct… |
| `00c3a9a0` | `PoolAllocSlot` | 🟢 | method | Allocates one 0x5c-byte slot from the pool sub-object: when the free list at this+8 is empty it calls the growth routine 00c3a820(0); pops the head, decrements this+0xc, and when the auto-shrink flag byte at this+0x10 i… |
| `00c3aa10` | `PoolFreeSlot` | 🟢 | method | Returns a slot to the pool: pushes it onto the free list at this+8 (slot+0 becomes the old head), increments this+0xc and, when the auto-shrink flag byte at this+0x10 is set, finds the owning block with 00c3a2f0 and inc… |
| `00c3aa80` | `SoeUtil::HashListMap::~HashListMap` | ✅ | dtor | Destructor of the (64,16) portal-category map: stores its own vtable, frees every slot of the free list at this+8 via 00c371a0 until the head is null, then frees the pool block-tree root at this+0x114 (children at +8/+0… |
| `00c3ab60` | `PoolAllocSlotThunk` | ✅ | virtual | Adjustor thunk in vtable slot 1: adds 0x114 to ECX and jumps to 00c3a9a0, forwarding the map's virtual slot-allocation call to the pool sub-object at +0x114. |
| `00c3ab70` | `PoolFreeSlotThunk` | ✅ | virtual | Adjustor thunk in vtable slot 2: adds 0x114 to ECX and jumps to 00c3aa10, forwarding the map's virtual slot-release call to the pool sub-object at +0x114. |
| `00c3aba0` | `SoeUtil::HashListMap::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of the portal-category map: runs the destructor 00c3aa80 and, when bit 0 of the flag byte is set, releases the object with _free; returns the object pointer. |
