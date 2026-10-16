# Class: SoccerPacketSetPlayerTeam

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b75370` | `— ⚠reject` | ⚪ | ctor | Constructor of the family-76 sub 6 packet SoccerPacketSetPlayerTeam: calls the shared base ctor FUN_00b730a0(6) and installs SoccerPacketSetPlayerTeam::vftable. |
| `00b753d0` | `SoccerPacketSetPlayerTeam::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object with _free. |
