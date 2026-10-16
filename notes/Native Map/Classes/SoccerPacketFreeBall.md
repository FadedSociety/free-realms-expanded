# Class: SoccerPacketFreeBall

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b75080` | `SoccerPacketFreeBall::Ctor` | ✅ | ctor | Constructor of the family-76 sub 0xc (12) packet SoccerPacketFreeBall: calls the shared base ctor FUN_00b730a0(0xc) and installs SoccerPacketFreeBall::vftable. |
| `00b750e0` | `SoccerPacketFreeBall::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object with _free. |
