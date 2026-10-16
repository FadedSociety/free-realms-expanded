# Class: EncounterMatchmaking::ClearMatchRequestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa1200` | `EncounterMatchmaking::ClearMatchRequestPacket::ClearMatchRequestPacket` | ✅ | ctor | Constructor: calls the matchmaking family constructor FUN_00a7f0d0 with 5, stores EncounterMatchmaking::ClearMatchRequestPacket::vftable at +0 and then constructs the embedded MatchmakingRequest member through FUN_0106f… |
| `00aa1260` | `EncounterMatchmaking::ClearMatchRequestPacket::~ClearMatchRequestPacket` | 🟢 | dtor | Destructor body: calls FUN_00a82b30 then rewrites the object vtable to EncounterMatchmaking::BaseMatchmakingPacket::vftable and BasePacket::vftable; referenced by one EH funclet. |
| `00aa19c0` | `EncounterMatchmaking::ClearMatchRequestPacket::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: calls the destructor body FUN_00aa1260 and, when bit 0 of the flags argument is set, calls _free on the object; returns the object pointer. |
