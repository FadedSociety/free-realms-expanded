# Class: VehicleRacePacketKickProxiedVehicleRace

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b99da0` | `VehicleRacePacketKickProxiedVehicleRace::~VehicleRacePacketKickProxiedVehicleRa…` | 🟢 | dtor | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9cae0` | `VehicleRacePacketKickProxiedVehicleRace::VehicleRacePacketKickProxiedVehicleRace` | ✅ | ctor | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(0xd) (inner opcode 33 subtype 13) and stores VehicleRacePacketKickProxiedVehicleRace::vftable into *this; returns this. |
| `00b9cb40` | `— ⚠reject` | ⚪ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
