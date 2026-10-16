# Class: VehicleRacePacketUpdateVehicleRaceServerData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b9bfb0` | `VehicleRacePacketUpdateVehicleRaceServerData::VehicleRacePacketUpdateVehicleRac…` | ✅ | ctor | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(6) (inner opcode 33 subtype 6) and stores VehicleRacePacketUpdateVehicleRaceServerData::vftable into *this; returns this. |
| `00b9c0c0` | `— ⚠reject` | ⚪ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
