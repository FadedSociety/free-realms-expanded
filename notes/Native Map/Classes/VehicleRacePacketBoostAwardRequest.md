# Class: VehicleRacePacketBoostAwardRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b9c7d0` | `VehicleRacePacketBoostAwardRequest::VehicleRacePacketBoostAwardRequest` | ✅ | ctor | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(0x16) (inner opcode 33 subtype 22) and stores VehicleRacePacketBoostAwardRequest::vftable into *this; returns this. |
| `00b9c830` | `VehicleRacePacketBoostAwardRequest::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
