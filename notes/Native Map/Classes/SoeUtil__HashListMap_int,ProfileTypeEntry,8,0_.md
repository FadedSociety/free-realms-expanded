# Class: SoeUtil::HashListMap<int,ProfileTypeEntry,8,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a16580` | `— ⚠reject` | ⚪ | ctor | Constructor of the derived map: writes the base HashListMap<int,ProfileTypeEntry,8,-1> vftable, zeroes the head, tail, count, bucket array and the fields up to +0x2c, then installs the derived vftable and zeroes the ext… |
| `00a17000` | `HashListMap_ProfileTypeEntry_8_0_Dtor` | ✅ | dtor | Destructor of the derived map: installs the derived vftable, drains the node list through FUN_008bf2b0, frees the binary tree at +0x34 with FUN_0073c670 for both children and FUN_00401c00 for the root, clears those fiel… |
| `00a170d0` | `HashListMap_ProfileTypeEntry_8_0_AdjustorSlot1` | ✅ | virtual | Eight-byte adjustor thunk (bytes 83c134 e9e8a0d2ff): adds 0x34 to ecx so the call targets the embedded allocator sub-object and tail-jumps to FUN_007411c0 with no frame of its own. |
| `00a170e0` | `HashListMap_ProfileTypeEntry_8_0_AdjustorSlot2` | ✅ | virtual | Eight-byte adjustor thunk, the twin of 00a170d0: adds 0x34 to ecx and tail-jumps to FUN_00741230. |
| `00a18780` | `HashListMap_ProfileTypeEntry_8_0_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): calls the derived destructor 00a17000 and frees the object with _free when bit 0 of the flag argument is set. |
