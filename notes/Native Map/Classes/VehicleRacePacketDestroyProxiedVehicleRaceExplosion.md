# Class: VehicleRacePacketDestroyProxiedVehicleRaceExplosion

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b9a090` | `Construct` | ✅ | ctor | Subtype constructor: calls the family base constructor 00b992d0 with subtype 0x1c (28) and then stores VehicleRacePacketDestroyProxiedVehicleRaceExplosion::vftable; returns this. |
| `00b9a0f0` | `Destruct → VehicleRacePacketDestroyProxiedVehicleRaceExplosion::~VehicleRacePac…` | 🟢 | dtor | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9cf00` | `VehicleRacePacketDestroyProxiedVehicleRaceExplosion::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
