# Class: SoccerPacketUpdateGameTime

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b738f0` | `— ⚠reject` | ⚪ | ctor | Constructor of the family-76 sub 0x17 (23) packet SoccerPacketUpdateGameTime: calls the shared base ctor FUN_00b730a0(0x17) and installs SoccerPacketUpdateGameTime::vftable. |
| `00b75230` | `SoccerPacketUpdateGameTime::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object with _free. |
