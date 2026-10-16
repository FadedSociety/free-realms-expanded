# Class: VehicleRacePacketDestroyProxiedVehicleRaceMissile

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b99e40` | `Construct → VehicleRacePacketDestroyProxiedVehicleRaceMissile::VehicleRacePacke…` | ✅ | ctor | Subtype constructor: calls the family base constructor 00b992d0 with subtype 0x19 (25) and then stores VehicleRacePacketDestroyProxiedVehicleRaceMissile::vftable; returns this. |
| `00b99ea0` | `Destruct → VehicleRacePacketDestroyProxiedVehicleRaceMissile::~VehicleRacePacke…` | 🟢 | dtor | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9cbb0` | `— ⚠reject` | ⚪ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
