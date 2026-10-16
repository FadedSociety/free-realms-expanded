# Class: VehicleDemolitionDerbyPacketSetDerbyConfig

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ea7e0` | `— ⚠reject` | ⚪ | ctor | Constructor: runs the derby base constructor with sub-opcode 0x13, stores the VehicleDemolitionDerbyPacketSetDerbyConfig vtable and constructs the embedded DemolitionDerbyConfig member with FUN_0105f170; returns this. |
| `009ea840` | `VehicleDemolitionDerbyPacketSetDerbyConfig_dtor` | 🟢 | dtor | Destructor: destroys the embedded DemolitionDerbyConfig member with FUN_0105f240 and then restores the BaseVehicleDemolitionDerbyPacket and BasePacket vtables. |
| `009ec730` | `VehicleDemolitionDerbyPacketSetDerbyConfig_scalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls the destructor body FUN_009ea840 (which destroys the embedded DemolitionDerbyConfig member through FUN_0105f240 and restores the two base vtables) and frees this through _free when bit … |
