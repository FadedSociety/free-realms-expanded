# Class: SoeUtil::List<ProxiedVehicleRace>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b9d3d0` | `SoeUtil::List_ProxiedVehicleRace::AllocNode` | ✅ | virtual | List node allocator (vtable slot 2): calls FUN_006fb9c0, then allocates through the current thread allocator FUN_006fb860 (using its virtual slot 1 when non-null), else through DAT_01bdf280's virtual slot 1, else FUN_00… |
| `00b9d410` | `SoeUtil::List_ProxiedVehicleRace::FreeNode` | ✅ | virtual | List node free (vtable slot 3): calls FreeMemory(param_1, 0x10) - FUN_00401c00 with alignment 16 - and returns. 18-byte leaf. |
| `00b9d720` | `SoeUtil::List_ProxiedVehicleRace::AddNew` | ✅ | method | Allocates a ProxiedVehicleRace node through list vtable slot 2, constructs it with 00b9b770, links it at the tail using node+0x1bb0/+0x1bb4, updates head when empty, and increments the list count. Returns void. |
| `00b9e560` | `SoeUtil::List_ProxiedVehicleRace::Remove` | 🟢 | method | Unlinks one node from the ProxiedVehicleRace list: fixes the neighbours through the next/prev links at node+0x1bb0 / node+0x1bb4 (updating head at this+4 and tail at this+8 when the node was at an end), decrements the c… |
| `00b9f110` | `SoeUtil::List_ProxiedVehicleRace::Clear` | 🟢 | method | Drains the ProxiedVehicleRace list at this+4: repeatedly takes the head node and passes it to 00b9e560 (remove) until the head is null. 27-byte loop. |
| `00b9f690` | `SoeUtil::List_ProxiedVehicleRace::Dtor` | ✅ | dtor | List destructor: stores SoeUtil::List<ProxiedVehicleRace,-1>::vftable at *this and then drains the list at +4 by removing the head node through 00b9e560 until empty; does not free the list object itself. |
| `00b9f830` | `SoeUtil::List_ProxiedVehicleRace::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: stores SoeUtil::List<ProxiedVehicleRace,-1>::vftable at *this, drains the list at +4 through 00b9e560 until empty, and when the low byte of param_2 is set calls _free(this); returns this. |
