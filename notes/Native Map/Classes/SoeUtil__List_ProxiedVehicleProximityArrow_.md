# Class: SoeUtil::List<ProxiedVehicleProximityArrow>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b9d6c0` | `SoeUtil::List_ProxiedVehicleProximityArrow::AllocNode` | ✅ | virtual | List node allocator (vtable slot 2) for the proximity-arrow list: allocates one 0x80-byte, 0x10-aligned node through the allocator chain (FUN_006fb9c0, FUN_006fb860 -> slot 1, DAT_01bdf280 -> slot 1, else FUN_006fb950(0… |
| `00b9d700` | `SoeUtil::List_ProxiedVehicleProximityArrow::FreeNode` | ✅ | virtual | List node free (vtable slot 3) for the proximity-arrow list: calls FreeMemory(param_1, 0x10) and returns. 18-byte leaf. |
| `00b9da50` | `SoeUtil::List_ProxiedVehicleProximityArrow::AddNew ? ⚠downgrade` | 🟡 | method | Allocates a ProxiedVehicleProximityArrow node through list vtable slot 2, constructs it with 00b9b970, links it at the tail using node+0x70/+0x74, updates head when empty, and increments the list count. Returns void. |
| `00b9ec50` | `SoeUtil::List_ProxiedVehicleProximityArrow::Remove` | 🟢 | method | Unlinks one node from the proximity-arrow list using the next/prev links at node+0x70 / node+0x74 (head at this+4, tail at this+8), decrements the count at this+0xc, destroys the node contents with FUN_00ce1310 and free… |
| `00b9f810` | `SoeUtil::List_ProxiedVehicleProximityArrow::Clear` | 🟢 | method | Drains the proximity-arrow list at this+4: repeatedly removes the head node through 00b9ec50 until the list is empty. 27-byte loop. |
