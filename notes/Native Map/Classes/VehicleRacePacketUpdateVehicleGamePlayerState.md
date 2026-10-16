# Class: VehicleRacePacketUpdateVehicleGamePlayerState

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b994b0` | `VehicleRacePacketUpdateVehicleGamePlayerState::~VehicleRacePacketUpdateVehicleG…` | 🟢 | dtor | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9bc80` | `VehicleRacePacketUpdateVehicleGamePlayerState::VehicleRacePacketUpdateVehicleGa…` | ✅ | ctor | Constructs an empty vehicle-race packet object in place: calls the family base constructor FUN_00b992d0(2) (inner opcode 33, subtype 2) and stores VehicleRacePacketUpdateVehicleGamePlayerState::vftable into *this; retur… |
| `00b9bce0` | `— ⚠reject` | ⚪ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
