# Class: VehicleRacePacketUpdateVehicleRaceClientData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b99660` | `VehicleRacePacketUpdateVehicleRaceClientData::~VehicleRacePacketUpdateVehicleRa…` | 🟢 | dtor | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9be30` | `VehicleRacePacketUpdateVehicleRaceClientData::VehicleRacePacketUpdateVehicleRac…` | ✅ | ctor | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(5) (inner opcode 33 subtype 5) and stores VehicleRacePacketUpdateVehicleRaceClientData::vftable into *this; returns this. |
| `00b9bf40` | `— ⚠reject` | ⚪ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
| `00b9dfd0` | `VehicleRacePacketUpdateVehicleRaceClientData::CopyCtor` | ✅ | ctor | Copy-constructs a 33/5 client-data packet: calls the base header copy 00b9d230, stores the class vftable, then copies the payload dwords +0xc through +0x54, the byte at +0x58 and the pair at +0x60/+0x64 from the source … |
| `00b9fb10` | `— ⚠reject` | ⚪ | method | Parses a 33/5 client-data payload: reads the base header via 00b9fab0, then six 12-byte vectors with FUN_009d4230 into +0xc, +0x18, +0x24, +0x30, +0x3c and +0x48, a 4-byte float at +0x54 (setting the reader's invalid-va… |
