# Class: SoeUtil::HashListMap<int,AppStoreBillboardPanelDefinition,16,4>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c3cfe0` | `SoeUtil::HashListMap::HashListMap` | ✅ | ctor | Constructor of the billboard-panel map: stores the base (16,-1) vtable, zeroes this+8/+0xc/+0x10 and the 0x40-byte bucket array at this+0x14 (16 buckets), stores the derived (16,4) vtable, zeroes the pool fields at +0x5… |
| `00c3d960` | `— ⚠reject` | ⚪ | method | Allocates one 0x6c-byte slot for the billboard-panel map: grows via 00c3d7e0(0) when the free list at this+8 is empty, pops the head, decrements this+0xc and, when the auto-shrink flag at this+0x10 is set and the slot l… |
| `00c3d9d0` | `PoolFreeSlot` | 🟢 | method | Returns a slot to the billboard-panel pool: pushes it onto the free list at this+8, increments this+0xc and, when the auto-shrink flag at this+0x10 is set, finds the owning block (00c3d2b0) and bumps its used counter at… |
| `00c3da40` | `PoolAllocSlotThunk` | ✅ | virtual | Adjustor thunk in vtable slot 1: adds 0x54 to ECX and jumps to 00c3d960, forwarding the map's virtual slot-allocation call to the pool sub-object at +0x54. |
| `00c3da50` | `PoolFreeSlotThunk` | ✅ | virtual | Adjustor thunk in vtable slot 2: adds 0x54 to ECX and jumps to 00c3d9d0, forwarding the map's virtual slot-release call to the pool sub-object at +0x54. |
| `00c3da60` | `— ⚠reject` | ⚪ | dtor | Deleting destructor of the (16,4) billboard-panel map: stores its vtable, frees the free-list at this+8 via 00c37bf0, releases the pool block-tree root at this+0x54 (children via 00c37ca0, memory via FreeMemory 00401c00… |
| `00c3db90` | `SoeUtil::HashListMap::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of the derived (16,4) billboard-panel map: runs the destructor 00c3da60 and, when bit 0 of the flag byte is set, releases the object with _free; returns the object pointer. |
