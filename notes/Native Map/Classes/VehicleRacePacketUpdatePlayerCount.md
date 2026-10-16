# Class: VehicleRacePacketUpdatePlayerCount

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b99500` | `VehicleRacePacketUpdatePlayerCount::VehicleRacePacketUpdatePlayerCount` | ✅ | ctor | Subtype constructor: calls the family base constructor 00b992d0 with subtype 3 and then stores VehicleRacePacketUpdatePlayerCount::vftable; returns this. |
| `00b99560` | `VehicleRacePacketUpdatePlayerCount::~VehicleRacePacketUpdatePlayerCount` | 🟢 | dtor | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9bd50` | `— ⚠reject` | ⚪ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
