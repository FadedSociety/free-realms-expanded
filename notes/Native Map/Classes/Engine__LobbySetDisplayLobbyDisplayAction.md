# Class: Engine::LobbySetDisplayLobbyDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127c9a0` | `Engine::LobbySetDisplayLobbyDisplayAction::ctor` | ✅ | ctor | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::LobbySetDisplayLobbyDisplayAction::vftable into *this; when the int argument is nonzero calls FUN_012ea300(this). |
| `0127ca30` | `Engine::LobbySetDisplayLobbyDisplayAction::DebugName` | ✅ | virtual | Returns the constant C string kSetDisplayLobbyDisplayAction : 0 : ... lobbyhandlers.cc:297 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of its vtable. |
| `0127ca40` | `Engine::LobbySetDisplayLobbyDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
