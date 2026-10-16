# Class: VehicleDemolitionDerbyPacketVehicleDemolitionDerbyCountdownStatus

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ebfd0` | `VehicleDemolitionDerbyPacketVehicleDemolitionDerbyCountdownStatus_ctor ⚠downgra…` | 🟢 | ctor | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 7 and stores the CountdownStatus vtable; returns this. |
| `009ec030` | `VehicleDemolitionDerbyPacketVehicleDemolitionDerbyCountdownStatus_scalarDeletin…` | ✅ | dtor | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set; returns this otherwise. |
| `009eedb0` | `VehicleDemolitionDerbyPacketVehicleDemolitionDerbyCountdownStatus_ReadBody` | 🟢 | method | Payload reader for subtype 7: header pair via 009ee770, a 64-bit key at +0x10/+0x14 and a dword at +0x18; consumes exactly the 12-byte captured payload plus the header pair. |
| `009efe10` | `VehicleDemolitionDerbyPacketVehicleDemolitionDerbyCountdownStatus_Deserialize` | 🟢 | function | Deserialises one subtype-7 payload through the 16-byte reader 009eedb0 into param_1 over (param_2, param_3); returns 1 only when no error was flagged and the buffer was consumed (or param_4 allows trailing bytes). |
