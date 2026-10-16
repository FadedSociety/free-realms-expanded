# Class: SoeUtil::List<ProxiedVehicleRaceMissile>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b9d570` | `SoeUtil::List_ProxiedVehicleRaceMissile::AllocNode` | ✅ | virtual | List node allocator (vtable slot 2) for the missile list: allocates one 0xa0-byte, 0x10-aligned node through the allocator chain (FUN_006fb9c0, FUN_006fb860 -> slot 1, DAT_01bdf280 -> slot 1, else FUN_006fb950(0xa0,0x10… |
| `00b9d5b0` | `SoeUtil::List_ProxiedVehicleRaceMissile::FreeNode` | ✅ | virtual | List node free (vtable slot 3) for the missile list: calls FreeMemory(param_1, 0x10) and returns. 18-byte leaf. |
| `00b9d870` | `SoeUtil::List_ProxiedVehicleRaceMissile::AddNew` | 🟢 | method | Allocates a ProxiedVehicleRaceMissile node through list vtable slot 2, constructs it with 00b9b850, links it at the tail using node+0x90/+0x94, updates head when empty, and increments the list count. Returns void. |
| `00b9e690` | `SoeUtil::List_ProxiedVehicleRaceMissile::Remove` | 🟢 | method | Unlinks one node from the missile list using the next/prev links at node+0x90 / node+0x94 (head at this+4, tail at this+8), decrements the count, destroys the node contents with FUN_00cdff80 and frees the node through t… |
| `00b9f150` | `SoeUtil::List_ProxiedVehicleRaceMissile::Clear` | 🟢 | method | Drains the missile list at this+4: repeatedly removes the head node through 00b9e690 until the list is empty. 27-byte loop. |
| `00b9f780` | `SoeUtil::List_ProxiedVehicleRaceMissile::Dtor` | ✅ | dtor | List destructor: stores SoeUtil::List<ProxiedVehicleRaceMissile,-1>::vftable at *this and drains the list at +4 by removing the head node through 00b9e690 until empty; does not free the list object. |
| `00b9f8b0` | `SoeUtil::List_ProxiedVehicleRaceMissile::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: stores SoeUtil::List<ProxiedVehicleRaceMissile,-1>::vftable at *this, drains the list at +4 through 00b9e690 until empty, and when the delete flag is set calls _free(this); returns this. |
