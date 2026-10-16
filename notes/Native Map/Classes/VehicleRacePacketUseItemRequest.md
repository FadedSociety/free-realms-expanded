# Class: VehicleRacePacketUseItemRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b9c620` | `VehicleRacePacketUseItemRequest::VehicleRacePacketUseItemRequest` | 🟢 | ctor | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(0x13) (inner opcode 33 subtype 19) and stores VehicleRacePacketUseItemRequest::vftable into *this; returns this. |
| `00b9c680` | `— ⚠reject` | ⚪ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
