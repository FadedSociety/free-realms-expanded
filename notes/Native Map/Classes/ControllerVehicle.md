# Class: ControllerVehicle

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cc1070` | `ControllerVehicle::Destroy` | ✅ | dtor | Restores the ControllerVehicle vtables at the primary object and offset 0xe0, then calls FUN_0094fab0 to release the base vehicle state. |
| `00cc10d0` | `ControllerVehicle::CallBaseThenVirtualA4` | ✅ | virtual | Calls FUN_0094e210, then invokes the object's vtable entry at offset 0xa4 with the constant stored at 0x01814c18. |
| `00cc14b0` | `ControllerVehicle::ControllerVehicle` | ✅ | ctor | Constructs the base vehicle, installs ControllerVehicle and physics-tick callback vtables, and initializes numerous controller state, transform, timing and flag fields from constants or zero. |
| `00cc16d0` | `ControllerVehicle::ScalarDeletingDtor` | ✅ | dtor | Calls the ControllerVehicle destructor and frees the object when the low bit of the flag argument is set; otherwise returns the object pointer. |
| `00cc1810` | `— ⚠reject` | ⚪ | virtual | Transforms four basis vectors through FUN_00770e70, scales their components with stored constants, and writes the resulting position and three basis vectors at offsets 0x110 through 0x11c. |
| `00cc1a00` | `ControllerVehicle::UpdateMovement` | ✅ | virtual | Accumulates the elapsed step, exits when the controller or movement config is absent, advances scalar movement state, computes normalized direction and position deltas, resolves collision traces, and writes the resultin… |
