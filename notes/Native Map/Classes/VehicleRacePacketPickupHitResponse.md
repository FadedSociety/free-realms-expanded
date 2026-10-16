# Class: VehicleRacePacketPickupHitResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b99990` | `VehicleRacePacketPickupHitResponse::VehicleRacePacketPickupHitResponse` | ✅ | ctor | Subtype constructor: calls the family base constructor 00b992d0 with subtype 0x12 (18) and then stores VehicleRacePacketPickupHitResponse::vftable; returns this. |
| `00b999f0` | `VehicleRacePacketPickupHitResponse::~VehicleRacePacketPickupHitResponse` | 🟢 | dtor | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9c5b0` | `— ⚠reject` | ⚪ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
