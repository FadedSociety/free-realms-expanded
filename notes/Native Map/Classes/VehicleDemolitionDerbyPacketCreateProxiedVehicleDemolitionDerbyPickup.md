# Class: VehicleDemolitionDerbyPacketCreateProxiedVehicleDemolitionDerbyPickup

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ecfd0` | `VehicleDemolitionDerbyPacketCreateProxiedVehicleDemolitionDerbyPickup_ctor` | ✅ | ctor | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 0xc, stores the CreateProxiedVehicleDemolitionDerbyPickup vtable and zeroes the member dword at +0x34 (param_1[0xd]); returns this. |
| `009ed030` | `VehicleDemolitionDerbyPacketCreateProxiedVehicleDemolitionDerbyPickup_scalarDel…` | ✅ | dtor | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set; returns this otherwise. |
| `009ee9e0` | `VehicleDemolitionDerbyPacketCreateProxiedVehicleDemolitionDerbyPickup_ReadBody` | 🟢 | method | Payload reader for subtype 0x0c: header pair via 009ee770, dwords into +0xc and +0x10, a 16-byte float4 block at +0x20 through FUN_008e2410 and floats at +0x30/+0x34; consumes exactly the 32-byte captured payload plus t… |
| `009efb00` | `VehicleDemolitionDerbyPacketCreateProxiedVehicleDemolitionDerbyPickup_Deseriali…` | 🟢 | function | Deserialises one subtype-0x0c payload through the 36-byte reader 009ee9e0 into param_1 over (param_2, param_3); returns 1 only when no error was flagged and the buffer was consumed (or param_4 allows trailing bytes). |
