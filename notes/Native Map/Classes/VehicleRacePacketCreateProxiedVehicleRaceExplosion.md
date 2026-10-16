# Class: VehicleRacePacketCreateProxiedVehicleRaceExplosion

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b9ee60` | `VehicleRacePacketCreateProxiedVehicleRaceExplosion::Ctor` | ✅ | ctor | Constructs the 33/27 create-proxied-explosion packet: calls the family base constructor FUN_00b992d0(0x1b), stores VehicleRacePacketCreateProxiedVehicleRaceExplosion::vftable into *this and initialises the embedded SoeU… |
| `00b9eed0` | `VehicleRacePacketCreateProxiedVehicleRaceExplosion::Dtor` | 🟢 | dtor | Destroys the 33/27 explosion packet in place: resets the embedded SoeUtil::IString<char> at +0x40 (releasing its refcounted buffer through virtual slot 2 when the length at +0x4c is positive), then stores BaseVehicleRac… |
| `00b9f320` | `VehicleRacePacketCreateProxiedVehicleRaceExplosion::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: calls 00b9eed0(this) to destroy the class (including its embedded IString) and, when the low byte of param_2 is set, calls _free(this); returns this. |
