# Class: SoeUtil::List<DerbyInfo,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ecb00` | `SoeUtil::List::AllocNode` | ✅ | virtual | Allocation virtual (vtable slot 2) of the DerbyInfo list: same manager probe, falling back to FUN_006fb950(0xb0, 8). |
| `009ecb40` | `SoeUtil::List::FreeNode` | ✅ | virtual | Freeing virtual (vtable slot 3) of the DerbyInfo list: 18-byte body forwarding its argument to FUN_00401c00 with alignment 8. |
| `009f0100` | `DerbyInfoList_RemoveNode` | 🟢 | method | Unlinks the node param_2 from the DerbyInfo list (head +4, tail +8, count +0xc; node next +0xa8, prev +0xac), decrements the count, destroys the node with 009ee630 and frees it through the list's vtable slot 3. |
| `009f0be0` | `SoeUtil_List_DerbyInfo_DtorBody` | ✅ | dtor | Stores the SoeUtil::List<DerbyInfo,-1> vtable at this+0, then while the head at this+4 is non-zero calls 009f0100(head) to unlink it. Does not free this. |
| `009f0c10` | `SoeUtil_List_DerbyInfo_ScalarDtor` | ✅ | dtor | Scalar deleting destructor (vftable slot [0]): stores the list vtable, unlinks nodes through 009f0100 while the head is non-zero (inlined dtor body), then calls the _free thunk on this when bit 0 of the argument is set;… |
