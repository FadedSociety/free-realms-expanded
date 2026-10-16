# Class: VehicleDemolitionDerbyPacketAnnouncerMessage

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ec750` | `VehicleDemolitionDerbyPacketAnnouncerMessage_ctor` | ✅ | ctor | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 0x14 and stores the AnnouncerMessage vtable; returns this. |
| `009ec7b0` | `VehicleDemolitionDerbyPacketAnnouncerMessage_scalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set; returns this otherwise. |
| `009eeeb0` | `VehicleDemolitionDerbyPacketAnnouncerMessage_ReadBody` | 🟢 | method | Payload reader for subtype 0x14: header pair via 009ee770, two dwords at +0xc/+0x10 and two 64-bit values at +0x18 and +0x20; consumes exactly the 24-byte captured payload plus the header pair. |
| `009eff60` | `VehicleDemolitionDerbyPacketAnnouncerMessage_Deserialize` | 🟢 | function | Deserialises one subtype-0x14 payload through the 28-byte reader 009eeeb0 into param_1 over (param_2, param_3); returns 1 only when no error was flagged and the buffer was consumed (or param_4 allows trailing bytes). |
