# Class: VehicleDemolitionDerbyPacketUpdateVehicleDemolitionDerbyData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ebe30` | `VehicleDemolitionDerbyPacketUpdateVehicleDemolitionDerbyData_ctor` | ✅ | ctor | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 5 and stores the VehicleDemolitionDerbyPacketUpdateVehicleDemolitionDerbyData vtable; returns this. |
| `009ebe90` | `VehicleDemolitionDerbyPacketUpdateVehicleDemolitionDerbyData_scalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set; returns this otherwise. |
| `009ed0b0` | `— ⚠reject` | ⚪ | ctor | Copy constructor: runs the base copy body 009ec880 on the source packet, stores the UpdateVehicleDemolitionDerbyData vtable, then copies the remaining 0x90 payload bytes (dwords from +0x10 to +0x9c plus bytes +0x8c..+0x… |
| `009ef5e0` | `VehicleDemolitionDerbyPacketUpdateVehicleDemolitionDerbyData_ReadBody` | 🟢 | method | Payload reader for subtype 5: header pair via 009ee770, a 64-bit key at +0x10/+0x14, six 16-byte float4 blocks at +0x20, +0x30, +0x40, +0x50, +0x60 and +0x70 read through FUN_008e2410, three floats at +0x80/+0x84/+0x88,… |
