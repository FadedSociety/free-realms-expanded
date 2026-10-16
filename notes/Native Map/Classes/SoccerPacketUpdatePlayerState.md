# Class: SoccerPacketUpdatePlayerState

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b746b0` | `SoccerPacketUpdatePlayerState::Ctor` | ✅ | ctor | Constructor of the family-76 sub 0xe (14) packet SoccerPacketUpdatePlayerState: calls the shared base ctor FUN_00b730a0(0xe) and installs SoccerPacketUpdatePlayerState::vftable. |
| `00b74710` | `SoccerPacketUpdatePlayerState::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object with _free. |
