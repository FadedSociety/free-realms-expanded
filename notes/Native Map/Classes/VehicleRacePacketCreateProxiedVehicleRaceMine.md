# Class: VehicleRacePacketCreateProxiedVehicleRaceMine

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b99f40` | `VehicleRacePacketCreateProxiedVehicleRaceMine::~VehicleRacePacketCreateProxiedV…` | 🟢 | dtor | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9ccf0` | `VehicleRacePacketCreateProxiedVehicleRaceMine::VehicleRacePacketCreateProxiedVe…` | ✅ | ctor | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(0x1d) (inner opcode 33 subtype 29) and stores VehicleRacePacketCreateProxiedVehicleRaceMine::vftable into *this; returns thi… |
| `00b9cd50` | `VehicleRacePacketCreateProxiedVehicleRaceMine::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
