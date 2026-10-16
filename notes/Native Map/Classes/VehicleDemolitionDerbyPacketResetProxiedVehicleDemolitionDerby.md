# Class: VehicleDemolitionDerbyPacketResetProxiedVehicleDemolitionDerby

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ec660` | `VehicleDemolitionDerbyPacketResetProxiedVehicleDemolitionDerby_ctor` | ✅ | ctor | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 0x12 and stores the ResetProxiedVehicleDemolitionDerby vtable; returns this. |
| `009ec6c0` | `VehicleDemolitionDerbyPacketResetProxiedVehicleDemolitionDerby_scalarDeletingDt…` | ✅ | dtor | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set; returns this otherwise. |
| `009ef520` | `VehicleDemolitionDerbyPacketResetProxiedVehicleDemolitionDerby_ReadBody` | 🟢 | method | Payload reader: header pair through 009ee770 and a 64-bit value into the record's +0x10/+0x14, with the usual zeroing and overrun flag; consumes exactly the 8-byte captured payload plus the header pair. |
| `009efe80` | `— ⚠reject` | ⚪ | function | Deserialises one subtype-0x12 payload through the 16-byte reader 009ef520 into param_1 over (param_2, param_3); returns 1 only when no error was flagged and the buffer was consumed (or param_4 allows trailing bytes). |
