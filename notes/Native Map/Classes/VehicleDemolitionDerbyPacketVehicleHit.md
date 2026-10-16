# Class: VehicleDemolitionDerbyPacketVehicleHit

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ec590` | `VehicleDemolitionDerbyPacketVehicleHit_ctor` | ✅ | ctor | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 0x10 and stores the VehicleHit vtable; returns this. |
| `009ec5f0` | `VehicleDemolitionDerbyPacketVehicleHit_scalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket then the BasePacket vtable and frees this through _free when bit 0 of param_2 is set; returns this otherwise. |
| `009eeba0` | `— ⚠reject` | ⚪ | method | Payload reader for subtype 0x10: header pair via 009ee770, then two 64-bit values (attacker at +0x10/+0x14, victim at +0x18/+0x1c) and a non-finite-checked float at +0x20; consumes exactly the 20-byte captured payload p… |
| `009efcc0` | `VehicleDemolitionDerbyPacketVehicleHit_Deserialize ? ⚠downgrade` | 🟡 | function | Deserialises one subtype-0x10 payload through the 24-byte reader 009eeba0 into param_1 over (param_2, param_3); returns 1 only when no error was flagged and the buffer was consumed (or param_4 allows trailing bytes). |
