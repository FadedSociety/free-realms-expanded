# Class: ProxiedVehicleRacePickup

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b9db50` | `ProxiedVehicleRacePickupDeletingDtor` | 🟢 | dtor | MSVC scalar deleting destructor for a ProxiedVehicleRacePickup: calls FUN_00cdfbb0(this) to destroy the object contents and, when the delete flag is set, calls _free(this); returns this. |
| `00cdfbb0` | `ProxiedVehicleRacePickup::DestructorBody` | ✅ | dtor | Restores the pickup vtable, releases the effect handle and references, resets fields on its owned object, and calls the stored object's virtual cleanup with its member value. |
| `00cdfc50` | `ProxiedVehicleRacePickup::Constructor` | ✅ | ctor | Stores the supplied fields, installs the ProxiedVehicleRacePickup vtable, and zeros the remaining owned-handle and state slots. |
| `00cdfc80` | `ProxiedVehicleRacePickup::ScalarDeletingDestructor` | ✅ | dtor | Calls the pickup destructor body, frees this when flag bit 0 is set, and otherwise returns this. |
