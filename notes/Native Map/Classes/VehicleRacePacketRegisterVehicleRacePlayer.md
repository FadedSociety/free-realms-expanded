# Class: VehicleRacePacketRegisterVehicleRacePlayer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b997a0` | `VehicleRacePacketRegisterVehicleRacePlayer::~VehicleRacePacketRegisterVehicleRa…` | 🟢 | dtor | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9c2d0` | `VehicleRacePacketRegisterVehicleRacePlayer::VehicleRacePacketRegisterVehicleRac…` | ✅ | ctor | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(9) (inner opcode 33 subtype 9) and stores VehicleRacePacketRegisterVehicleRacePlayer::vftable into *this; returns this. |
| `00b9c330` | `— ⚠reject` | ⚪ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
