# Class: VehicleRacePacketDestroyProxiedVehicleRace

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b99d50` | `VehicleRacePacketDestroyProxiedVehicleRace::~VehicleRacePacketDestroyProxiedVeh…` | 🟢 | dtor | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9ca10` | `VehicleRacePacketDestroyProxiedVehicleRace::VehicleRacePacketDestroyProxiedVehi…` | ✅ | ctor | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(0xc) (inner opcode 33 subtype 12) and stores VehicleRacePacketDestroyProxiedVehicleRace::vftable into *this; returns this. |
| `00b9ca70` | `VehicleRacePacketDestroyProxiedVehicleRace::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
