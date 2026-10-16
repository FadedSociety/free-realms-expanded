# Class: VehicleDemolitionDerbyPacketUpdatePlayerCount

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ea1b0` | `VehicleDemolitionDerbyPacketUpdatePlayerCount_ctor` | ✅ | ctor | Constructor: runs the derby base constructor with sub-opcode 3 and stores the VehicleDemolitionDerbyPacketUpdatePlayerCount vtable. |
| `009ebd50` | `VehicleDemolitionDerbyPacketUpdatePlayerCount_scalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set; returns this otherwise. |
| `009ef330` | `VehicleDemolitionDerbyPacketUpdatePlayerCount_ReadBody` | 🟢 | method | Payload reader: header pair through 009ee770 (reader in param_1, record in param_2 here) and one dword from the stream into the record's +0xc, zeroing it and setting stream+0x10 on overrun; consumes exactly the 4-byte c… |
| `009ef9b0` | `VehicleDemolitionDerbyPacketUpdatePlayerCount_Deserialize` | 🟢 | function | When param_1 and param_2 are non-null, builds a 0x14-byte stack reader record with four dwords (buffer, length, cursor, end) and two status bytes, then calls 009ef330 to parse into param_1. Returns 1 when no error flag … |
