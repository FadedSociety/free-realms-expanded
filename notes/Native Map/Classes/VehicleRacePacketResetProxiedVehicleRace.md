# Class: VehicleRacePacketResetProxiedVehicleRace

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b9a190` | `VehicleRacePacketResetProxiedVehicleRace::~VehicleRacePacketResetProxiedVehicle…` | 🟢 | dtor | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9d040` | `VehicleRacePacketResetProxiedVehicleRace::VehicleRacePacketResetProxiedVehicleR…` | ✅ | ctor | Constructs the 33/33 reset-proxied-vehicle packet: calls the family base constructor FUN_00b992d0(0x21), stores VehicleRacePacketResetProxiedVehicleRace::vftable into *this, then writes the sentinel dword DAT_01b83900 (… |
| `00b9d110` | `VehicleRacePacketResetProxiedVehicleRace::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
| `00ba0260` | `ParseResetProxiedVehicleRacePacket` | 🟢 | method | Parses a 33/33 reset payload: header via 00b9fab0, u64 at +0x10/+0x14, then four 12-byte vectors via FUN_009d4230 into +0x18, +0x24, +0x30 and +0x3c; every read is bounds-checked with the truncated flag on underflow. |
