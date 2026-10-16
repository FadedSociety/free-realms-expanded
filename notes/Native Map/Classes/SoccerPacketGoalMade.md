# Class: SoccerPacketGoalMade

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b73360` | `— ⚠reject` | ⚪ | ctor | Constructor of the family-76 sub 2 packet SoccerPacketGoalMade: calls the shared base ctor FUN_00b730a0(2) and installs SoccerPacketGoalMade::vftable into the object. |
| `00b749f0` | `SoccerPacketGoalMade::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object with _free. |
