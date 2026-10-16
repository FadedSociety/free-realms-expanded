# Class: SoeUtil::List<ItemDefinitionMetaData,32>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c72420` | `SoeUtilListItemDef32_Ctor` | ✅ | ctor | Stores the List<ItemDefinitionMetaData,-1> base vtable, zeroes words [1], [2], [3], then stores SoeUtil::List<ItemDefinitionMetaData,32>::vftable and calls 00c704c0 (pool init). Returns this. |
| `00c72e50` | `SoeUtilListItemDef32::Dtor` | ✅ | dtor | Stores the List<ItemDefinitionMetaData,32> vtable, drains the base list through 00c716f0, frees the tree block at [param_1+0x10] through 00c71340 and 00401c00 (alignment 4), zeroes [4]-[7], then stores the List<ItemDefi… |
| `00c72f00` | `ThisAdjustJump_00c72d70 ? → SoeUtil::List<ItemDefinitionMetaData,32>::Slot2_Chu…` | 🟡 | function | Adds 0x10 to ECX (this) and jumps to 00c72d70 (bytes 83 c1 10 e9 rel32). A this-adjusting jump thunk: the List<32> slot [2] entry at +0x10 of the object, with no body. |
| `00c72f10` | `ThisAdjustJump_00c72de0 ? → SoeUtil::List<ItemDefinitionMetaData,32>::Slot3_Chu…` | 🟡 | function | Adds 0x10 to ECX (this) and jumps to 00c72de0 (bytes 83 c1 10 e9 rel32). A this-adjusting jump thunk with no body of its own. |
| `00c73cb0` | `SoeUtil::List_ItemDefinitionMetaData_32::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls the body FUN_00c72e50 on this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
