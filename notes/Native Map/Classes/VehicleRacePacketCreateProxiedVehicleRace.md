# Class: VehicleRacePacketCreateProxiedVehicleRace

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b99cf0` | `VehicleRacePacketCreateProxiedVehicleRace::~VehicleRacePacketCreateProxiedVehic…` | 🟢 | dtor | Destructor of the create-proxied-vehicle packet: runs FUN_01056260 (which destroys a VehicleConfig and its friction-material hash map), then stores BaseVehicleRacePacket::vftable and BasePacket::vftable. |
| `00b9c910` | `VehicleRacePacketCreateProxiedVehicleRace::VehicleRacePacketCreateProxiedVehicl…` | ✅ | ctor | Constructs the 33/11 create-proxied-vehicle packet: calls the family base constructor FUN_00b992d0(0xb), stores VehicleRacePacketCreateProxiedVehicleRace::vftable into *this, calls FUN_010569f0 (embedded vehicle-definit… |
| `00b9c9f0` | `VehicleRacePacketCreateProxiedVehicleRace::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: calls FUN_00b99cf0(this) (the out-of-line base destructor that resets the base vtables) and, when the low byte of param_2 is set, calls _free(this); returns this. |
| `00ba7370` | `VehicleRacePacketCreateProxiedVehicleRace::ReadPayload` | ✅ | method | Reads the vehicle-race header, a variable-length vehicle definition, an 8-byte owner id, four 12-byte vectors into fields at this+0xe1c through this+0xe40, then two one-byte tail fields at this+0xe4c and this+0xe18; bou… |
