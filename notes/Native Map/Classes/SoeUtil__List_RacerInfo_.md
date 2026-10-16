# Class: SoeUtil::List<RacerInfo>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b9d440` | `SoeUtil::List_RacerInfo::AllocNode` | ✅ | virtual | List node allocator (vtable slot 2) for the RacerInfo list: allocates one 0x28-byte, 8-aligned node through the same allocator chain (FUN_006fb9c0, FUN_006fb860 -> slot 1, DAT_01bdf280 -> slot 1, else FUN_006fb950(0x28,… |
| `00b9d480` | `SoeUtil::List_RacerInfo::FreeNode` | ✅ | virtual | List node free (vtable slot 3) for the RacerInfo list: calls FreeMemory(param_1, 8) and returns. 18-byte leaf. |
| `00b9f6c0` | `SoeUtil::List_RacerInfo::AddNew` | 🟢 | method | Allocates a RacerInfo node through list vtable slot 2, constructs it with 00b9f1b0, links it at the tail using node+0x20/+0x24, updates head when empty, and increments the list count. Returns void. |
| `00b9fa20` | `SoeUtil::List_RacerInfo::Remove` | 🟢 | method | Unlinks one node from the racer-info list using the next/prev links at node+0x20 / node+0x24 (head at this+4, tail at this+8), decrements the count at this+0xc, destroys the node contents with 00b9eb30 and frees the nod… |
