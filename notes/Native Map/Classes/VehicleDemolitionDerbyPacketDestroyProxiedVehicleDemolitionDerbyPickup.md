# Class: VehicleDemolitionDerbyPacketDestroyProxiedVehicleDemolitionDerbyPickup

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ea450` | `VehicleDemolitionDerbyPacketDestroyProxiedVehicleDemolitionDerbyPickup_ctor` | ✅ | ctor | Constructor: runs the derby base constructor with sub-opcode 0xd and stores the destroy-proxied-pickup vtable. |
| `009ec0a0` | `VehicleDemolitionDerbyPacketDestroyProxiedVehicleDemolitionDerbyPickup_scalarDe…` | ✅ | dtor | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set; returns this otherwise. |
| `009ef3d0` | `VehicleDemolitionDerbyPacketDestroyProxiedVehicleDemolitionDerbyPickup_ReadBody` | 🟢 | method | Payload reader: header pair through 009ee770 and one dword from the stream into the record's +0xc (zeroed with the overrun flag on failure); consumes exactly the 4-byte captured payload plus the header pair. |
| `009efb70` | `VehicleDemolitionDerbyPacketDestroyProxiedVehicleDemolitionDerbyPickup_Deserial…` | 🟢 | function | Deserialises one subtype-0x0d payload through the 8-byte reader 009ef3d0 into param_1 over (param_2, param_3); returns 1 only when no error was flagged and the buffer was consumed (or param_4 allows trailing bytes). |
