# Class: VehicleRacePacketUpdateVehicleGameState

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b99400` | `VehicleRacePacketUpdateVehicleGameState::VehicleRacePacketUpdateVehicleGameStat…` | 🟢 | ctor | Subtype constructor: calls the family base constructor 00b992d0 with subtype 1 and then stores VehicleRacePacketUpdateVehicleGameState::vftable; returns this. |
| `00b99460` | `VehicleRacePacketUpdateVehicleGameState::~VehicleRacePacketUpdateVehicleGameSta…` | 🟢 | dtor | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9bc10` | `— ⚠reject` | ⚪ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this (base-destruction chain), and when the low byte of param_2 is non-zero calls _free(this); the flag-0 path re… |
