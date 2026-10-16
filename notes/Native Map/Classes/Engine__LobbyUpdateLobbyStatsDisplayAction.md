# Class: Engine::LobbyUpdateLobbyStatsDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127cc50` | `Engine::LobbyUpdateLobbyStatsDisplayAction::ctor` | ✅ | ctor | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::LobbyUpdateLobbyStatsDisplayAction::vftable into *this; when the int argument is nonzero calls FUN_012ea300(this). |
| `0127cce0` | `Engine::LobbyUpdateLobbyStatsDisplayAction::DebugName` | ✅ | virtual | Returns the constant C string kUpdateLobbyStatsDisplayAction : 0 : ... lobbyhandlers.cc:323 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of its vtable. |
| `0127ccf0` | `Engine::LobbyUpdateLobbyStatsDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `0127fee0` | `Engine::LobbyUpdateLobbyStatsDisplayAction::Vslot2` | 🟢 | virtual | Virtual slot [2]. Calls FUN_012d46a0(1) and FUN_012d2200(1) for a user count, builds the tag QueryLobbyStats with FUN_012ec220, sets keys GroupID (constant 1) and UserCount, then calls FUN_012e5730 and FUN_012e5820. Ret… |
