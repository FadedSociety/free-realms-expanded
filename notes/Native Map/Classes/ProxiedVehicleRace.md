# Class: ProxiedVehicleRace

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b9daf0` | `ProxiedVehicleRaceDeletingDtor` | 🟢 | dtor | MSVC scalar deleting destructor for a ProxiedVehicleRace object: calls FUN_00cdf8e0(this) to destroy the object contents, and when the low byte of param_2 is set calls _free(this); returns this. |
| `00cdf8e0` | `ProxiedVehicleRace::DestructorBody` | ✅ | dtor | Restores three ProxiedVehicleRace vtables, clears state bytes, restores embedded packet base vtables, and destroys the owned vehicle-race state. |
| `00cdf9b0` | `ProxiedVehicleRace::Constructor` | ✅ | ctor | Initializes its base data, installs three ProxiedVehicleRace vtables, constructs an embedded VehicleRacePacketUpdateVehicleRaceClientData object, and clears four flags. |
| `00cdfa70` | `ProxiedVehicleRace::ScalarDeletingDestructor` | ✅ | dtor | Calls the ProxiedVehicleRace destructor body, frees this when flag bit 0 is set, and otherwise returns this. |
