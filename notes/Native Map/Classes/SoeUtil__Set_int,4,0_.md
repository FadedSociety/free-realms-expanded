# Class: SoeUtil::Set<int,4,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a17c10` | `— ⚠reject` | ⚪ | ctor | Constructor: writes the Set<int,-1,0> vftable, zeroes the root and count, installs the Set<int,4,0> vftable, zeroes the four inline free-list slots, points the embedded node pointers at each other (this+0x20, +0x28, +0x… |
| `00a18660` | `— ⚠reject` | ⚪ | dtor | Destructor of the derived set: installs the derived vftable, frees the node tree at +4 by recursing through FUN_007a2470 and dispatching virtual slot 3 of the set for the root, frees the second tree at +0xc with 00a14c9… |
| `00a18760` | `Set_int_4_0_AdjustorSlot2` | ✅ | virtual | Eight-byte adjustor thunk (bytes 83c10c e918feffff): adds 0xc to ecx, retargeting the call to the pool sub-object at this+0xc, and tail-jumps to the small-object pool allocator 00a18580. |
| `00a18770` | `Set_AllocatorInterface_AdjustorSlot0` | ✅ | virtual | Eight-byte adjustor thunk: adds 0xc to ecx and tail-jumps to the pool block free routine 00a185f0; it is the entry of the allocator interface vtable referenced as vtable[3] of the set. |
| `00a18820` | `Set_int_4_0_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): calls the set destructor 00a18660 and frees the object with _free when bit 0 of the flag argument is set. |
