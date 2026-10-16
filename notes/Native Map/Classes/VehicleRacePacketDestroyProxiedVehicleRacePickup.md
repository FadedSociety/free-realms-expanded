# Class: VehicleRacePacketDestroyProxiedVehicleRacePickup

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b99890` | `Construct` | ✅ | ctor | Subtype constructor: calls the family base constructor 00b992d0 with subtype 0x10 (16) and then stores VehicleRacePacketDestroyProxiedVehicleRacePickup::vftable; returns this. |
| `00b998f0` | `Destruct → VehicleRacePacketDestroyProxiedVehicleRacePickup::~VehicleRacePacket…` | 🟢 | dtor | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9c470` | `— ⚠reject` | ⚪ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
