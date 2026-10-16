# Class: VehicleRacePacketCreateProxiedVehicleRacePickup

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b99840` | `Destruct → VehicleRacePacketCreateProxiedVehicleRacePickup::~VehicleRacePacketC…` | 🟢 | dtor | Destructor body: stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9de30` | `VehicleRacePacketCreateProxiedVehicleRacePickup::VehicleRacePacketCreateProxied…` | ✅ | ctor | Constructs an empty vehicle-race packet object: calls the family base constructor FUN_00b992d0(0xf) (inner opcode 33 subtype 15) and stores VehicleRacePacketCreateProxiedVehicleRacePickup::vftable into *this; returns th… |
| `00b9de90` | `VehicleRacePacketCreateProxiedVehicleRacePickup::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: writes BaseVehicleRacePacket::vftable and then BasePacket::vftable into *this, and when the low byte of param_2 is non-zero calls _free(this); otherwise returns this. |
| `00ba0070` | `ParseCreatePickupPacket` | 🟢 | method | Parses a 33/15 create-proxied-pickup payload: header via 00b9fab0, then u32 at +0xc, u32 at +0x10, a 16-byte float4 via FUN_008e2410 into +0x20, a float at +0x30 (invalid-value flag on NaN) and a final u32 at +0x34; all… |
