# Class: SoeUtil::List<ProxiedVehicleRaceExplosion>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b9d650` | `SoeUtil::List_ProxiedVehicleRaceExplosion::AllocNode` | ✅ | virtual | List node allocator (vtable slot 2) for the explosion list: allocates one 0x60-byte, 0x10-aligned node through the allocator chain (FUN_006fb9c0, FUN_006fb860 -> slot 1, DAT_01bdf280 -> slot 1, else FUN_006fb950(0x60,0x… |
| `00b9d690` | `SoeUtil::List_ProxiedVehicleRaceExplosion::FreeNode` | ✅ | virtual | List node free (vtable slot 3) for the explosion list: calls FreeMemory(param_1, 0x10) and returns. 18-byte leaf. |
| `00b9d9b0` | `SoeUtil::List_ProxiedVehicleRaceExplosion::AddNew` | 🟢 | method | Allocates a ProxiedVehicleRaceExplosion node through list vtable slot 2, constructs it with 00b9b910, links it at the tail using node+0x50/+0x54, updates head when empty, and increments the list count. Returns void. |
| `00b9e7d0` | `SoeUtil::List_ProxiedVehicleRaceExplosion::Remove` | 🟢 | method | Unlinks one node from the explosion list using the next/prev links at node+0x50 / node+0x54 (head at this+4, tail at this+8), decrements the count, destroys the node contents with FUN_00ce0dd0 and frees the node through… |
| `00b9f190` | `SoeUtil::List_ProxiedVehicleRaceExplosion::Clear` | 🟢 | method | Drains the explosion list at this+4: repeatedly removes the head node through 00b9e7d0 until the list is empty. 27-byte loop. |
| `00b9f7e0` | `SoeUtil::List_ProxiedVehicleRaceExplosion::Dtor` | ✅ | dtor | List destructor: stores SoeUtil::List<ProxiedVehicleRaceExplosion,-1>::vftable at *this and drains the list at +4 by removing the head node through 00b9e7d0 until empty; does not free the list object. |
| `00b9f930` | `SoeUtil::List_ProxiedVehicleRaceExplosion::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: stores SoeUtil::List<ProxiedVehicleRaceExplosion,-1>::vftable at *this, drains the list at +4 through 00b9e7d0 until empty, and when the delete flag is set calls _free(this); returns thi… |
