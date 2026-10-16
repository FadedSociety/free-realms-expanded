# Class: SoccerPacketRegisterPlayer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b74b30` | `SoccerPacketRegisterPlayer::Ctor ⚠downgrade` | 🟢 | ctor | Constructor of the family-76 sub 3 packet SoccerPacketRegisterPlayer: calls the shared base ctor FUN_00b730a0(3) and installs SoccerPacketRegisterPlayer::vftable. |
| `00b74b90` | `SoccerPacketRegisterPlayer::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object with _free. |
