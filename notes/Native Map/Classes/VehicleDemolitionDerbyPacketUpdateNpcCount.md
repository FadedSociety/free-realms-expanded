# Class: VehicleDemolitionDerbyPacketUpdateNpcCount

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ea260` | `VehicleDemolitionDerbyPacketUpdateNpcCount_ctor` | ✅ | ctor | Constructor: runs the derby base constructor with sub-opcode 4 and stores the VehicleDemolitionDerbyPacketUpdateNpcCount vtable. |
| `009ebdc0` | `VehicleDemolitionDerbyPacketUpdateNpcCount_scalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set; returns this otherwise. |
| `009ef370` | `VehicleDemolitionDerbyPacketUpdateNpcCount_ReadBody` | 🟢 | method | Payload reader: header pair through 009ee770 and one dword from the stream into the record's +0xc, with the usual zeroing and overrun flag; consumes exactly the 4-byte captured payload plus the header pair. |
| `009efa20` | `VehicleDemolitionDerbyPacketUpdateNpcCount_Deserialize` | 🟢 | function | Deserialises one subtype-4 payload through the record reader 009ef370 into param_1 over (param_2, param_3) and returns 1 only when no error was flagged and the buffer was consumed (or param_4 allows trailing bytes). |
