# Class: VehicleRacePacketDestroyProxiedVehicleRaceMine

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b99f90` | `VehicleRacePacketDestroyProxiedVehicleRaceMine::VehicleRacePacketDestroyProxied…` | ✅ | ctor | Subtype constructor: calls the family base constructor 00b992d0 with subtype 0x1e (30) and then stores VehicleRacePacketDestroyProxiedVehicleRaceMine::vftable; returns this. |
| `00b99ff0` | `VehicleRacePacketDestroyProxiedVehicleRaceMine::~VehicleRacePacketDestroyProxie…` | 🟢 | dtor | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9cdc0` | `VehicleRacePacketDestroyProxiedVehicleRaceMine::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
