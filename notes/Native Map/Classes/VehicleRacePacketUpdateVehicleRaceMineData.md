# Class: VehicleRacePacketUpdateVehicleRaceMineData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b9a040` | `VehicleRacePacketUpdateVehicleRaceMineData::~VehicleRacePacketUpdateVehicleRace…` | 🟢 | dtor | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9ce30` | `VehicleRacePacketUpdateVehicleRaceMineData::VehicleRacePacketUpdateVehicleRaceM…` | ✅ | ctor | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(0x1f) (inner opcode 33 subtype 31) and stores VehicleRacePacketUpdateVehicleRaceMineData::vftable into *this; returns this. |
| `00b9ce90` | `VehicleRacePacketUpdateVehicleRaceMineData::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
