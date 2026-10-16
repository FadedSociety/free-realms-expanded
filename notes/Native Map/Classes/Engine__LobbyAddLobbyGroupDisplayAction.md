# Class: Engine::LobbyAddLobbyGroupDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127c460` | `Engine::LobbyAddLobbyGroupDisplayAction::ctor` | ✅ | ctor | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::LobbyAddLobbyGroupDisplayAction::vftable into *this; when the int argument is nonzero calls FUN_012ea300(this). |
| `0127c4f0` | `Engine::LobbyAddLobbyGroupDisplayAction::DebugName` | ✅ | virtual | Returns the constant C string kAddLobbyGroupDisplayAction : 0 : ... lobbyhandlers.cc:187 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of its vtable. |
| `0127c500` | `Engine::LobbyAddLobbyGroupDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `0127f580` | `Engine::LobbyAddLobbyGroupDisplayAction::Vslot2` | 🟢 | virtual | Virtual slot [2]. Calls FUN_012e0ba0 and returns 1 when FUN_0127a580() is null. Otherwise reads ValueData entries 0, 1 and 2 as integers (type check, 0 on failure). When entry 1 == 1 it calls the manager's virtual slot … |
