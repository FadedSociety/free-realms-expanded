# Class: VehicleDemolitionDerbyPacketRegisterVehicleDemolitionDerbyPlayer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ebf00` | `VehicleDemolitionDerbyPacketRegisterVehicleDemolitionDerbyPlayer_ctor` | ✅ | ctor | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 6 and stores the RegisterVehicleDemolitionDerbyPlayer vtable; returns this. |
| `009ebf60` | `VehicleDemolitionDerbyPacketRegisterVehicleDemolitionDerbyPlayer_scalarDeleting…` | ✅ | dtor | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set; returns this otherwise. |
