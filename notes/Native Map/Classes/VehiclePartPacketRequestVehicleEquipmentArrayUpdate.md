# Class: VehiclePartPacketRequestVehicleEquipmentArrayUpdate

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b86b30` | `— ⚠reject` | ⚪ | ctor | Constructor of the equipment-array update request packet: runs the base packet constructor FUN_009d2370 with sub-opcode constant 3, installs the class vtable and initializes the extra fields through FUN_0101cd40. |
| `00b86ea0` | `VehiclePartPacketRequestVehicleEquipmentArrayUpdate::ScalarDeletingDtor` | ✅ | dtor | Vtable slot 0 of the equipment-array update request packet: resets the base vtables (BaseVehiclePartPacket then BasePacket) and frees the object through _free when bit 0 of param_2 is set. |
