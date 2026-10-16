# Class: VehicleDemolitionDerbyPacketUpdateVehicleDemolitionDerbyStateData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ec2b0` | `VehicleDemolitionDerbyPacketUpdateVehicleDemolitionDerbyStateData_ctor` | ✅ | ctor | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 0xb and stores the UpdateVehicleDemolitionDerbyStateData vtable; returns this. |
| `009ec310` | `VehicleDemolitionDerbyPacketUpdateVehicleDemolitionDerbyStateData_scalarDeletin…` | ✅ | dtor | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set; returns this otherwise. |
| `009ee850` | `VehicleDemolitionDerbyPacketUpdateVehicleDemolitionDerbyStateData_ReadBody` | 🟢 | method | Payload reader for subtype 0x0b: header pair via 009ee770, a 64-bit key into +0x10/+0x14, six dwords into +0x18..+0x2c, three bool bytes into +0x30..+0x32 and four float4-checked floats into +0x34..+0x43; every overrun … |
| `009efa90` | `— ⚠reject` | ⚪ | function | Deserialises one subtype-0x0b payload through the 51-byte reader 009ee850 into param_1 over (param_2, param_3); returns 1 only when no error was flagged and the buffer was consumed (or param_4 allows trailing bytes). |
