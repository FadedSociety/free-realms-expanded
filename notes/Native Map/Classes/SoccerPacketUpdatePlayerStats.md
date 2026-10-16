# Class: SoccerPacketUpdatePlayerStats

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b74780` | `— ⚠reject` | ⚪ | ctor | Constructor of the family-76 sub 0xf (15) packet SoccerPacketUpdatePlayerStats: calls the shared base ctor FUN_00b730a0(0xf) and installs SoccerPacketUpdatePlayerStats::vftable. |
| `00b747e0` | `SoccerPacketUpdatePlayerStats::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object with _free. |
