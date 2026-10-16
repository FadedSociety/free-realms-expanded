# Class: EncounterMatchmaking::AddMatchRequestResponsePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa1140` | `EncounterMatchmaking::AddMatchRequestResponsePacket::AddMatchRequestResponsePac…` | ✅ | ctor | Constructor: calls the matchmaking family constructor FUN_00a7f0d0 with 4, stores EncounterMatchmaking::AddMatchRequestResponsePacket::vftable at +0, then constructs the embedded MatchmakingRequest member through FUN_01… |
| `00aa11a0` | `EncounterMatchmaking::AddMatchRequestResponsePacket::~AddMatchRequestResponsePa…` | 🟢 | dtor | Destructor body: calls FUN_00a82b30 (the embedded request/member teardown) then rewrites the object vtable to EncounterMatchmaking::BaseMatchmakingPacket::vftable and BasePacket::vftable; runs under an SEH frame and is … |
| `00aa19a0` | `EncounterMatchmaking::AddMatchRequestResponsePacket::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: calls the destructor body FUN_00aa11a0 and, when bit 0 of the flags argument is set, calls _free on the object; returns the object pointer. |
