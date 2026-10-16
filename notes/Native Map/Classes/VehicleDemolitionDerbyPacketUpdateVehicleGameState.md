# Class: VehicleDemolitionDerbyPacketUpdateVehicleGameState

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ea0b0` | `VehicleDemolitionDerbyPacketUpdateVehicleGameState_ctor` | ✅ | ctor | Constructor: runs the derby base constructor with sub-opcode 1 and stores the VehicleDemolitionDerbyPacketUpdateVehicleGameState vtable; returns this. |
| `009ebc10` | `— ⚠reject` | ⚪ | dtor | Scalar deleting destructor: restores the two base vtables in order (BaseVehicleDemolitionDerbyPacket::vftable then BasePacket::vftable) and, when bit 0 of param_2 is set, frees this through _free (does not return); retu… |
| `009ef570` | `VehicleDemolitionDerbyPacketUpdateVehicleGameState_ReadBody` | 🟢 | method | Payload reader: header pair through 009ee770 and one dword from the stream into the record's +0xc, with the usual zeroing and overrun flag; consumes exactly the 4-byte captured payload plus the header pair. |
| `009efef0` | `VehicleDemolitionDerbyPacketUpdateVehicleGameState_Deserialize` | 🟢 | function | Deserialises one subtype-1 payload through the 8-byte reader 009ef570 into param_1 over (param_2, param_3); returns 1 only when no error was flagged and the buffer was consumed (or param_4 allows trailing bytes). |
