# Class: ProxiedVehicleRaceMine

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b9dc10` | `ProxiedVehicleRaceMineDeletingDtor` | 🟢 | dtor | MSVC scalar deleting destructor for a ProxiedVehicleRaceMine: calls FUN_00ce0820(this) to destroy the object contents and, when the delete flag is set, calls _free(this); returns this. |
| `00ce0820` | `ProxiedVehicleRaceMine::~ProxiedVehicleRaceMine` | ✅ | dtor | Stores ProxiedVehicleRaceMine::vftable at this; calls vtbl+8 of the object at [this+0x1c] with [this+0x14]; if [this+0xb4] is nonzero it sets byte +0x4d of the object at [this+0xb8] to 1, calls that object's vtbl+0xc wi… |
| `00ce0970` | `ProxiedVehicleRaceMine::ProxiedVehicleRaceMine` | ✅ | ctor | Thiscall constructor with two stack arguments (ret 8): stores the second stack argument (param_3) at this+0x10, stores ProxiedVehicleRaceMine::vftable at this, zeroes this+0x14, stores the first stack argument (param_2)… |
| `00ce09a0` | `ProxiedVehicleRaceMine::ScalarDeletingDtor` | ✅ | dtor | Thiscall scalar deleting destructor (ret 4): calls the body 00ce0820, and when bit 0 of the flag argument is set calls _free(this) (the call returns, the epilogue returns this in eax); returns this. |
