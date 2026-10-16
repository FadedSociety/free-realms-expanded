# Class: VehicleRacePacketUpdateVehicleRaceProgressData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b9a140` | `VehicleRacePacketUpdateVehicleRaceProgressData::~VehicleRacePacketUpdateVehicle…` | 🟢 | dtor | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9cf70` | `— ⚠reject` | ⚪ | ctor | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(0xe) (inner opcode 33 subtype 14) and stores VehicleRacePacketUpdateVehicleRaceProgressData::vftable into *this; returns thi… |
| `00b9cfd0` | `VehicleRacePacketUpdateVehicleRaceProgressData::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
| `00b9fc80` | `ParseProgressDataPacket` | ✅ | method | Parses a 33/14 progress payload in this read order: header via 00b9fab0, then u64 guid into +0x10/+0x14, u32 into +0x38, floats into +0x3c, +0x40, u32 into +0x44, a byte (stored as a bool at +0x48), u32 at +0x4c, floats… |
