# Class: VehicleRacePacketBoostAwardResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b99c40` | `VehicleRacePacketBoostAwardResponse::VehicleRacePacketBoostAwardResponse` | ✅ | ctor | Subtype constructor: calls the family base constructor 00b992d0 with subtype 0x17 (23) and then stores VehicleRacePacketBoostAwardResponse::vftable; returns this. |
| `00b99ca0` | `VehicleRacePacketBoostAwardResponse::~VehicleRacePacketBoostAwardResponse` | 🟢 | dtor | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9c8a0` | `— ⚠reject` | ⚪ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
