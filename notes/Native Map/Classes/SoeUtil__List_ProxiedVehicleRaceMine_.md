# Class: SoeUtil::List<ProxiedVehicleRaceMine>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b9d5e0` | `SoeUtil::List_ProxiedVehicleRaceMine::AllocNode` | ✅ | virtual | List node allocator (vtable slot 2) for the mine list: allocates one 0xd0-byte, 0x10-aligned node through the allocator chain (FUN_006fb9c0, FUN_006fb860 -> slot 1, DAT_01bdf280 -> slot 1, else FUN_006fb950(0xd0,0x10)). |
| `00b9d620` | `SoeUtil::List_ProxiedVehicleRaceMine::FreeNode` | ✅ | virtual | List node free (vtable slot 3) for the mine list: calls FreeMemory(param_1, 0x10) and returns. 18-byte leaf. |
| `00b9d910` | `SoeUtil::List_ProxiedVehicleRaceMine::AddNew` | 🟢 | method | Allocates a ProxiedVehicleRaceMine node through list vtable slot 2, constructs it with 00b9b8b0, links it at the tail using node+0xc0/+0xc4, updates head when empty, and increments the list count. Returns void. |
| `00b9e730` | `SoeUtil::List_ProxiedVehicleRaceMine::Remove` | 🟢 | method | Unlinks one node from the mine list using the next/prev links at node+0xc0 / node+0xc4 (head at this+4, tail at this+8), decrements the count, destroys the node contents with FUN_00ce0820 and frees the node through the … |
| `00b9f170` | `SoeUtil::List_ProxiedVehicleRaceMine::Clear` | 🟢 | method | Drains the mine list at this+4: repeatedly removes the head node through 00b9e730 until the list is empty. 27-byte loop. |
| `00b9f7b0` | `SoeUtil::List_ProxiedVehicleRaceMine::Dtor` | ✅ | dtor | List destructor: stores SoeUtil::List<ProxiedVehicleRaceMine,-1>::vftable at *this and drains the list at +4 by removing the head node through 00b9e730 until empty; does not free the list object. |
| `00b9f8f0` | `SoeUtil::List_ProxiedVehicleRaceMine::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: stores SoeUtil::List<ProxiedVehicleRaceMine,-1>::vftable at *this, drains the list at +4 through 00b9e730 until empty, and when the delete flag is set calls _free(this); returns this. |
