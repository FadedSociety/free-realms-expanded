# Class: VehicleRacePacketCreateProxiedVehicleRaceMissile

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b99df0` | `Destruct → VehicleRacePacketCreateProxiedVehicleRaceMissile::~VehicleRacePacket…` | 🟢 | dtor | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9df00` | `VehicleRacePacketCreateProxiedVehicleRaceMissile::Ctor` | ✅ | ctor | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(0x18) (inner opcode 33 subtype 24) and stores VehicleRacePacketCreateProxiedVehicleRaceMissile::vftable into *this; returns … |
| `00b9df60` | `VehicleRacePacketCreateProxiedVehicleRaceMissile::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
