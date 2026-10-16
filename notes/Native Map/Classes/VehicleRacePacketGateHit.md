# Class: VehicleRacePacketGateHit

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b9c130` | `VehicleRacePacketGateHit::VehicleRacePacketGateHit` | ✅ | ctor | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(7) (inner opcode 33 subtype 7) and stores VehicleRacePacketGateHit::vftable into *this; returns this. |
| `00b9c190` | `— ⚠reject` | ⚪ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
