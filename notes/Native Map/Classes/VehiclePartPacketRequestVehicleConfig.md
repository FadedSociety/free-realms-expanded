# Class: VehiclePartPacketRequestVehicleConfig

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b86a80` | `VehiclePartPacketRequestVehicleConfig::VehiclePartPacketRequestVehicleConfig` | ✅ | ctor | Constructor of the vehicle-config request packet: runs the base packet constructor FUN_009d2370 with sub-opcode constant 1 and installs VehiclePartPacketRequestVehicleConfig::vftable. |
| `00b86e30` | `VehiclePartPacketRequestVehicleConfig::ScalarDeletingDtor` | ✅ | dtor | Vtable slot 0 of the vehicle-config request packet: resets the object to BaseVehiclePartPacket::vftable then BasePacket::vftable and frees the packet through the _free thunk when bit 0 of param_2 is set. |
