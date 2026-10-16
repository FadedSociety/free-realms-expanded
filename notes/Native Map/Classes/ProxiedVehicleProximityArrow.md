# Class: ProxiedVehicleProximityArrow

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b9dd70` | `ProxiedVehicleProximityArrowDeletingDtor` | 🟢 | dtor | MSVC scalar deleting destructor for a ProxiedVehicleProximityArrow: calls FUN_00ce1310(this) to destroy the object contents and, when the delete flag is set, calls _free(this); returns this. |
| `00ce12c0` | `— ⚠reject` | ⚪ | ctor | Thiscall constructor with two stack arguments: stores the second argument at this+0x18, ProxiedVehicleProximityArrow::vftable at this, the dword at its +4 at this+0x1c, zeroes +0x14, +0x20, +0x24 and +0x28, stores the f… |
| `00ce1310` | `ProxiedVehicleProximityArrow::~ProxiedVehicleProximityArrow` | ✅ | dtor | Stores ProxiedVehicleProximityArrow::vftable at this; when [this+0x14] is nonnull calls vtbl+8 of the object at [this+0x1c] with the dword at [[this+0x14]+0x268]. |
| `00ce1340` | `ProxiedVehicleProximityArrow::ScalarDeletingDtor` | ✅ | dtor | Thiscall scalar deleting destructor (ret 4): stores ProxiedVehicleProximityArrow::vftable, performs the same body inline (vtbl+8 of the object at [this+0x1c] with the dword at [[this+0x14]+0x268] when [this+0x14] is non… |
