# Class: VehicleRacePacketUpdateNpcCount

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b995b0` | `VehicleRacePacketUpdateNpcCount::VehicleRacePacketUpdateNpcCount` | ✅ | ctor | Subtype constructor: calls the family base constructor 00b992d0 with subtype 4 and then stores VehicleRacePacketUpdateNpcCount::vftable; returns this. |
| `00b99610` | `VehicleRacePacketUpdateNpcCount::~VehicleRacePacketUpdateNpcCount` | 🟢 | dtor | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9bdc0` | `VehicleRacePacketUpdateNpcCount::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
