# Class: VehicleRacePacketPickupHitRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b9c4e0` | `VehicleRacePacketPickupHitRequest::VehicleRacePacketPickupHitRequest` | ✅ | ctor | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(0x11) (inner opcode 33 subtype 17) and stores VehicleRacePacketPickupHitRequest::vftable into *this; returns this. |
| `00b9c540` | `— ⚠reject` | ⚪ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
