# Class: VehicleRacePacketUseItemResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b99a90` | `VehicleRacePacketUseItemResponse::VehicleRacePacketUseItemResponse` | ✅ | ctor | Subtype constructor: calls the family base constructor 00b992d0 with subtype 0x14 (20) and then stores VehicleRacePacketUseItemResponse::vftable; returns this. |
| `00b99af0` | `VehicleRacePacketUseItemResponse::~VehicleRacePacketUseItemResponse` | 🟢 | dtor | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9c6f0` | `— ⚠reject` | ⚪ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
