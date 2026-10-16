# Class: VehicleDemolitionDerbyPacketPickupHitResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ec450` | `VehicleDemolitionDerbyPacketPickupHitResponse_ctor ⚠downgrade` | 🟢 | ctor | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 0xf and stores the PickupHitResponse vtable; returns this. |
| `009ec4b0` | `VehicleDemolitionDerbyPacketPickupHitResponse_scalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set; returns this otherwise. |
| `009eec60` | `VehicleDemolitionDerbyPacketPickupHitResponse_ReadBody` | 🟢 | method | Payload reader for subtype 0x0f: header pair via 009ee770, a 64-bit key at +0x10/+0x14, a dword at +0x18 and three floats at +0x1c/+0x20/+0x24; consumes exactly the 24-byte captured payload plus the header pair. |
| `009efd30` | `VehicleDemolitionDerbyPacketPickupHitResponse_Deserialize` | 🟢 | function | Deserialises one subtype-0x0f payload through the 28-byte reader 009eec60 into param_1 over (param_2, param_3); returns 1 only when no error was flagged and the buffer was consumed (or param_4 allows trailing bytes). |
