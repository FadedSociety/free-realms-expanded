# Class: ProxiedVehicleRaceMissile

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b9dbb0` | `ProxiedVehicleRaceMissileDeletingDtor` | 🟢 | dtor | MSVC scalar deleting destructor for a ProxiedVehicleRaceMissile: calls FUN_00cdff80(this) to destroy the object contents and, when the delete flag is set, calls _free(this); returns this. |
| `00cdff80` | `ProxiedVehicleRaceMissile::DestructorBody` | ✅ | dtor | Restores the missile vtable, releases its effect and object handles, resets owner fields, and decrements the retained reference at +0x88. |
| `00ce00e0` | `ProxiedVehicleRaceMissile::Constructor` | ✅ | ctor | Stores the supplied fields, installs the ProxiedVehicleRaceMissile vtable, and zeros the remaining owner, handle, and state slots. |
| `00ce0120` | `ProxiedVehicleRaceMissile::ScalarDeletingDestructor` | ✅ | dtor | Calls the missile destructor body, frees this when flag bit 0 is set, and otherwise returns this. |
