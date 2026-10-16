# Class: VehiclePartPacketRequestSelectVehicle

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b86be0` | `VehiclePartPacketRequestSelectVehicle::VehiclePartPacketRequestSelectVehicle` | ✅ | ctor | Constructor of the select-vehicle request packet: runs the base packet constructor FUN_009d2370 with sub-opcode constant 5, installs the class vtable and zeroes the two payload fields at +0xc and +0x10. |
| `00b86f10` | `VehiclePartPacketRequestSelectVehicle::ScalarDeletingDtor` | ✅ | dtor | Vtable slot 0 of the select-vehicle request packet: resets the base vtables and frees the object through _free when bit 0 of param_2 is set. |
