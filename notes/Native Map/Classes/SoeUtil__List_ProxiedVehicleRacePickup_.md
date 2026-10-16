# Class: SoeUtil::List<ProxiedVehicleRacePickup>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b9d500` | `SoeUtil::List_ProxiedVehicleRacePickup::AllocNode` | ✅ | virtual | List node allocator (vtable slot 2) for the pickup list: allocates one 0x2c-byte, 4-aligned node through the allocator chain (FUN_006fb9c0, FUN_006fb860 -> slot 1, DAT_01bdf280 -> slot 1, else FUN_006fb950(0x2c,4)). |
| `00b9d540` | `SoeUtil::List_ProxiedVehicleRacePickup::FreeNode` | ✅ | virtual | List node free (vtable slot 3) for the pickup list: calls FreeMemory(param_1, 4) and returns. 18-byte leaf. |
| `00b9d7d0` | `SoeUtil::List_ProxiedVehicleRacePickup::AddNew` | 🟢 | method | Allocates a ProxiedVehicleRacePickup node through list vtable slot 2, constructs it with 00b9b7f0, links it at the tail using node+0x24/+0x28, updates head when empty, and increments the list count. Returns void. |
| `00b9e600` | `SoeUtil::List_ProxiedVehicleRacePickup::Remove` | 🟢 | method | Unlinks one node from the pickup list using the next/prev links at node+0x24 / node+0x28 (head at this+4, tail at this+8), decrements the count, destroys the node contents with FUN_00cdfbb0 and frees the node through th… |
| `00b9f130` | `SoeUtil::List_ProxiedVehicleRacePickup::Clear` | 🟢 | method | Drains the pickup list at this+4: repeatedly removes the head node through 00b9e600 until the list is empty. 27-byte loop. |
| `00b9f750` | `SoeUtil::List_ProxiedVehicleRacePickup::Dtor` | ✅ | dtor | List destructor: stores SoeUtil::List<ProxiedVehicleRacePickup,-1>::vftable at *this and drains the list at +4 by removing the head node through 00b9e600 until empty; does not free the list object. |
| `00b9f870` | `SoeUtil::List_ProxiedVehicleRacePickup::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: stores SoeUtil::List<ProxiedVehicleRacePickup,-1>::vftable at *this, drains the list at +4 through 00b9e600 until empty, and when the delete flag is set calls _free(this); returns this. |
