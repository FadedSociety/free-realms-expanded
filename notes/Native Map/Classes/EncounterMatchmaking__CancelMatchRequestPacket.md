# Class: EncounterMatchmaking::CancelMatchRequestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a7fc60` | `EncounterMatchmaking::CancelMatchRequestPacket::CancelMatchRequestPacket` | ✅ | ctor | Constructor: calls the matchmaking packet base constructor 00a7f0d0 with sub-opcode 6 and installs the CancelMatchRequestPacket vtable. |
| `00a7fcc0` | `EncounterMatchmaking::CancelMatchRequestPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: rewrites the vtable pointer to BaseMatchmakingPacket::vftable and then BasePacket::vftable and frees the object when bit 0 of param_2 is set. |
