# Class: SoeUtil::List<SoeUtil::EmbeddedList<TargetProcessorEventHandler,4>,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00acb260` | `SoeUtil::List::AllocNode` | ✅ | virtual | Allocates a 20-byte list node: FUN_006fb9c0 then FUN_006fb860 select the current allocator object; when one exists its virtual slot +4 is invoked with (0x14,4); otherwise the global allocator object at DAT_01bdf280 is u… |
| `00acb2a0` | `SoeUtil::List::FreeNode` | ✅ | virtual | Calls FUN_00401c00(param_1,4), cleans the two pushed arguments, and returns with ret 4. This is a normal call wrapper for the free routine, not a tail jump. |
| `00acb4d0` | `ListAppendNode` | 🟢 | method | Obtains a node through the list virtual slot +8, clears dwords at node offsets 0, 4, 8 and 0x10, links node+0x0c to the old tail, updates the old tail or empty-list head, sets list[1] to the new tail and increments list… |
| `00acb610` | `ListRemoveTailNode` | 🟢 | method | Removes the tail node list[1]: when the node has no successor (+0x10 clear) the tail becomes its predecessor (+0xc); otherwise the successor's predecessor link is rewired; when the node has no predecessor the head at li… |
| `00acb660` | `ListRemoveAllNodes` | 🟢 | method | Drains the list: repeatedly takes the tail node from list[1], unlinks it from its neighbours (updating the head at list[2] when it was first and the tail when needed), decrements the count at list[3] and frees the node … |
| `00acb700` | `SoeUtil::List::Destructor` | ✅ | dtor | Stores the specialized SoeUtil::List vtable at *this and jumps to 00acb660. The target function drains the list; this 11-byte thunk does not contain that body. |
| `00acb740` | `SoeUtil::List::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: reinstalls the list vtable, removes all nodes through FUN_00acb660 and frees the object with _free when bit 0 of the flag argument is set. |
