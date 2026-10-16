# Class: SoccerPacketSetSoccerPlayerOneTimeStats

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b74e10` | `— ⚠reject` | ⚪ | ctor | Constructor of the family-76 sub 0x15 (21) packet SoccerPacketSetSoccerPlayerOneTimeStats: calls the shared base ctor FUN_00b730a0(0x15) and installs SoccerPacketSetSoccerPlayerOneTimeStats::vftable. |
| `00b74e70` | `SoccerPacketSetSoccerPlayerOneTimeStats::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object with _free. |
