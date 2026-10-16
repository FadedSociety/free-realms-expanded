# Class: VehicleRacePacketItemExpired

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b99b40` | `VehicleRacePacketItemExpired::VehicleRacePacketItemExpired` | ✅ | ctor | Subtype constructor: calls the family base constructor 00b992d0 with subtype 0x15 (21) and then stores VehicleRacePacketItemExpired::vftable; returns this. |
| `00b99ba0` | `VehicleRacePacketItemExpired::~VehicleRacePacketItemExpired` | 🟢 | dtor | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9c760` | `— ⚠reject` | ⚪ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
