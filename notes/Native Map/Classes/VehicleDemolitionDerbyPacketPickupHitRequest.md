# Class: VehicleDemolitionDerbyPacketPickupHitRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ec380` | `VehicleDemolitionDerbyPacketPickupHitRequest_ctor` | ✅ | ctor | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 0xe and stores the PickupHitRequest vtable; returns this. |
| `009ec3e0` | `VehicleDemolitionDerbyPacketPickupHitRequest_scalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set; returns this otherwise. |
