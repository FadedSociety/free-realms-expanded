# Class: SoeUtil::Array<GuiDsTreeNode*,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0099b0f0` | `ArrayGuiDsTreeNodePtrSlot1CapacityAlloc` | 🟢 | virtual | Virtual slot [1] of SoeUtil::Array<GuiDsTreeNode*,0,1>, thiscall with three stack args (count, out-pointer for the chosen count, flag byte). Let c=[this+0xc]. If the flag is zero: if c < count, the new count is count*5/… |
| `0099b190` | `— ⚠reject` | ⚪ | virtual | Virtual slot [2] of SoeUtil::Array<GuiDsTreeNode*,0,1>, one stack arg (buffer), 18 bytes. Calls 00401c00(buffer,4) (free with alignment 4) and returns void. |
| `0099b1b0` | `— ⚠reject` | ⚪ | virtual | Virtual slot [3] of SoeUtil::Array<GuiDsTreeNode*,0,1>, thiscall with two stack args (other array pointer param_2, byte flag). If the flag is nonzero, calls vtable slot +0xc of *param_2 and returns. Otherwise exchanges … |
| `0099dad0` | `ArrayGuiDsTreeNodeDtor` | ✅ | dtor | Restores the array vtable, clears its size, frees its backing allocation with element alignment 4, and nulls the allocation pointer. |
| `0099e620` | `ArrayGuiDsTreeNodeScalarDeletingDtor` | ✅ | virtual | Restores the typed array vtable, frees its backing storage with alignment 4, clears the pointer and count, and optionally frees the object itself. |
