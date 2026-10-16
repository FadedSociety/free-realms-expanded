# Class: SoeUtil::HashListMap<int,GameCommerce::StoreShortcutDefinition,64,16>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c3ce60` | `SoeUtil::HashListMap::HashListMap` | ✅ | ctor | Constructor of the store-shortcut map: stores the base (64,-1) vtable, zeroes this+8/+0xc/+0x10 and the 0x100-byte bucket array at this+0x14 (64 buckets), stores the derived (64,16) vtable and constructs the pool sub-ob… |
| `00c3cee0` | `PoolAllocSlotThunk` | ✅ | virtual | Adjustor thunk in vtable slot 1: adds 0x114 to ECX and jumps to 00c3cc80, forwarding the store-shortcut map's virtual slot-allocation call to the pool sub-object at +0x114 (same pool geometry as the category-group map, … |
| `00c3cef0` | `PoolFreeSlotThunk` | ✅ | virtual | Adjustor thunk in vtable slot 2: adds 0x114 to ECX and jumps to 00c3ccf0, forwarding the store-shortcut map's virtual slot-release call to the pool sub-object at +0x114. |
| `00c3cf00` | `— ⚠reject` | ⚪ | dtor | Deleting destructor of the (64,16) shortcut map: stores its vtable, frees the free-list at this+8 via 00c37ac0, releases the pool block-tree root at this+0x114 (children via 00c37a80, memory via FreeMemory 00401c00), ze… |
| `00c3db70` | `SoeUtil::HashListMap::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of the derived (64,16) shortcut map: runs the destructor 00c3cf00 and, when bit 0 of the flag byte is set, releases the object with _free; returns the object pointer. |
