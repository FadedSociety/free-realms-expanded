# Class: VehicleDemolitionDerbyPacketKickProxiedVehicleDemolitionDerby

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ec110` | `VehicleDemolitionDerbyPacketKickProxiedVehicleDemolitionDerby_ctor` | ✅ | ctor | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 9 and stores the KickProxiedVehicleDemolitionDerby vtable; returns this. |
| `009ec170` | `VehicleDemolitionDerbyPacketKickProxiedVehicleDemolitionDerby_scalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set; returns this otherwise. |
| `009ef410` | `VehicleDemolitionDerbyPacketKickProxiedVehicleDemolitionDerby_ReadBody` | 🟢 | method | Payload reader: header pair through 009ee770 and a 64-bit value into the record's +0x10/+0x14 (both zeroed with the overrun flag on failure). |
| `009efbe0` | `— ⚠reject` | ⚪ | function | Deserialises one subtype-9 payload through the 16-byte reader 009ef410 into param_1 over (param_2, param_3); returns 1 only when no error was flagged and the buffer was consumed (or param_4 allows trailing bytes). |
