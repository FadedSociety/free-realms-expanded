# Class: SoccerPacketHighArcBall

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b73550` | `— ⚠reject` | ⚪ | ctor | Constructor of the family-76 sub 0x12 packet SoccerPacketHighArcBall: calls the shared base ctor FUN_00b730a0(0x12) and installs SoccerPacketHighArcBall::vftable. |
| `00b74da0` | `SoccerPacketHighArcBall::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object with _free. |
