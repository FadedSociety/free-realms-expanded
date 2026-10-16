# Class: SoeUtil::Set<GuiPropertySet*,-1,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bddc0` | `GuiPropertySetPointerSet::AllocateNode` | ✅ | virtual | Allocates a 0x14-byte node aligned to 4 after trying the thread and global allocators. |
| `008c7be0` | `SoeUtil_SetGuiPropertySet_Dtor` | ✅ | dtor | Stores SoeUtil::Set<GuiPropertySet*,-1,0>::vftable at this+0. If the node pointer at this+4 is nonzero it calls FUN_0085bf70 on the words at node+8 and node+0xc, then calls vslot 3 (+0xc) of this with the node pointer. … |
| `008c93b0` | `SoeUtil_SetGuiPropertySet_ScalarDeletingDtor` | ✅ | dtor | Stores the Set<GuiPropertySet*,-1,0> vtable at this+0, runs the node teardown (FUN_0085bf70 on both child words, FUN_008bde00 on the node) when set, and frees this with _free when bit 0 of the flag byte is set. Returns … |
| `008ebca0` | `GuiPropertySetCollectionDtor` | ✅ | dtor | Calls 008ebc30 with the embedded map pointer at this +0x24 in ECX, releases the two values in the current set node, calls the set-node cleanup virtual at vtable offset +0xc, then clears the set fields. |
