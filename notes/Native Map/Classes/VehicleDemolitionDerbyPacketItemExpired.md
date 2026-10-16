# Class: VehicleDemolitionDerbyPacketItemExpired

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ea690` | `VehicleDemolitionDerbyPacketItemExpired_ctor` | ✅ | ctor | Constructor: runs the derby base constructor with sub-opcode 0x11 and stores the VehicleDemolitionDerbyPacketItemExpired vtable. |
| `009ec520` | `VehicleDemolitionDerbyPacketItemExpired_scalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set; returns this otherwise. |
| `009ef4d0` | `VehicleDemolitionDerbyPacketItemExpired_ReadBody ? ⚠downgrade` | 🟡 | method | Payload reader: header pair through 009ee770 and one dword from the stream into the record's +0xc, with the usual zeroing and overrun flag. |
| `009efda0` | `VehicleDemolitionDerbyPacketItemExpired_Deserialize ? ⚠downgrade` | 🟡 | function | Deserialises one subtype-0x11 payload through the 8-byte reader 009ef4d0 into param_1 over (param_2, param_3); returns 1 only when no error was flagged and the buffer was consumed (or param_4 allows trailing bytes). |
