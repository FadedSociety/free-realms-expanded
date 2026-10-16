# Class: VehicleDemolitionDerbyPacketDestroyProxiedVehicleDemolitionDerby

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ec1e0` | `VehicleDemolitionDerbyPacketDestroyProxiedVehicleDemolitionDerby_ctor` | ✅ | ctor | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 8 and stores the DestroyProxiedVehicleDemolitionDerby vtable; returns this. |
| `009ec240` | `VehicleDemolitionDerbyPacketDestroyProxiedVehicleDemolitionDerby_scalarDeleting…` | ✅ | dtor | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set; returns this otherwise. |
| `009ef460` | `VehicleDemolitionDerbyPacketDestroyProxiedVehicleDemolitionDerby_ReadBody` | 🟢 | method | Payload reader: header pair through 009ee770 and a 64-bit value into the record's +0x10/+0x14, with the usual zeroing and overrun flag. |
| `009efc50` | `— ⚠reject` | ⚪ | function | Deserializes subtype 8 into param_1 over (param_2,param_3) through reader 009ef460, which consumes the 4-byte opcode/subtype header and one 8-byte key (12 bytes total). Returns 1 only when no error was flagged and the b… |
