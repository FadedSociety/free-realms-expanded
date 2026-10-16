# Class: SoccerPacketBallBounce

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b73840` | `SoccerPacketBallBounce::Ctor` | ✅ | ctor | Constructor of the family-76 sub 0x19 (25) packet SoccerPacketBallBounce: calls the shared base ctor FUN_00b730a0(0x19) and installs SoccerPacketBallBounce::vftable. |
| `00b751c0` | `SoccerPacketBallBounce::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object with _free. |
