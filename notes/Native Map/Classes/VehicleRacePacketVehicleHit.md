# Class: VehicleRacePacketVehicleHit

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b99750` | `VehicleRacePacketVehicleHit::~VehicleRacePacketVehicleHit` | 🟢 | dtor | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9c200` | `VehicleRacePacketVehicleHit::VehicleRacePacketVehicleHit` | ✅ | ctor | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(8) (inner opcode 33 subtype 8) and stores VehicleRacePacketVehicleHit::vftable into *this; returns this. |
| `00b9c260` | `— ⚠reject` | ⚪ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
| `00b9ffb0` | `ParseVehicleHitPacket` | 🟢 | method | Parses a 33/8 vehicle-hit payload: header via 00b9fab0, then u64 at +0x10/+0x14, u64 at +0x18/+0x1c and a 4-byte float at +0x20 (setting the reader's invalid-value byte at +0x11 when the float is NaN); each read is boun… |
