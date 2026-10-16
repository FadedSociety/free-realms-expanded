# Class: VehicleRacePacketSetRaceConfig

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b9a1e0` | `VehicleRacePacketSetRaceConfig::VehicleRacePacketSetRaceConfig` | ✅ | ctor | Subtype constructor: calls the family base constructor 00b992d0 with subtype 0x22 (34), stores VehicleRacePacketSetRaceConfig::vftable and then constructs the embedded RaceConfig member through FUN_0107ccc0; returns thi… |
| `00b9a240` | `VehicleRacePacketSetRaceConfig::~VehicleRacePacketSetRaceConfig` | 🟢 | dtor | Destructor of the set-race-config packet: destroys the embedded RaceConfig member with FUN_0107cdd0, then stores BaseVehicleRacePacket::vftable and BasePacket::vftable. |
| `00b9d180` | `VehicleRacePacketSetRaceConfig::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: calls FUN_00b9a240(this) (the out-of-line base destructor that restores the base vtables) and, when the low byte of param_2 is set, calls _free(this); returns this. |
| `00ba0680` | `ParseSetRaceConfigBody` | ✅ | method | Thiscall parser of an 84-byte body with no header read: u32 to +0x30, u32 to +0x34, float to +0x38 (sets reader NaN byte +0x11 when NaN), four float4 via ReadFloat4 (008e2410) to +0x40/+0x50/+0x60/+0x70, then u32 to +0x… |
