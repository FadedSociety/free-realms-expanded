# Class: SoccerPacketUpdateSoccerBall

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b74cd0` | `SoccerPacketUpdateSoccerBall::Ctor` | ✅ | ctor | Constructor of the family-76 sub 4 packet SoccerPacketUpdateSoccerBall: calls the shared base ctor FUN_00b730a0(4) and installs SoccerPacketUpdateSoccerBall::vftable. |
| `00b74d30` | `SoccerPacketUpdateSoccerBall::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object with _free. |
