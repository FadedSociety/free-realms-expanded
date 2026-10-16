# Class: SoeUtil::Set<int,1024,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c73020` | `SoeUtilSet1024_Ctor` | ✅ | ctor | Stores the Set<int,-1,0> base vtable, zeroes words [1] and [2], then stores SoeUtil::Set<int,1024,0>::vftable and calls 00bbb460 (set initialisation). Returns this. |
| `00c730a0` | `— ⚠reject` | ⚪ | dtor | Stores the Set<int,1024,0> vtable, drains and frees the node chain at [param_1+4] through 007a2470 and the virtual at slot 3 (00c731b0 thunk), then releases the tree block at [param_1+0xc] through 00bbdd90 and 00401c00 … |
| `00c731a0` | `ThisAdjustJump_00bc01f0 ? → SoeUtil::Set<int,1024,0>::Slot2_ThisAdjustJump_00bc…` | 🟡 | function | Adds 0xc to ECX (this) and jumps to 00bc01f0 (bytes 83 c1 0c e9 rel32). A this-adjusting jump thunk with no body. |
| `00c73cf0` | `SoeUtil::Set_int_1024_0::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls the body FUN_00c730a0 on this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
