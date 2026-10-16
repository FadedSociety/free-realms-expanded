# Class: SoccerPacketUpdateGameState

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b752a0` | `SoccerPacketUpdateGameState::Ctor` | ✅ | ctor | Constructor of the family-76 sub 5 packet SoccerPacketUpdateGameState: calls the shared base ctor FUN_00b730a0(5) and installs SoccerPacketUpdateGameState::vftable. |
| `00b75300` | `SoccerPacketUpdateGameState::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object with _free. |
