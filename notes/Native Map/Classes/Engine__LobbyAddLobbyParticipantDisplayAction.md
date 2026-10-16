# Class: Engine::LobbyAddLobbyParticipantDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127c5b0` | `Engine::LobbyAddLobbyParticipantDisplayAction::ctor` | ✅ | ctor | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::LobbyAddLobbyParticipantDisplayAction::vftable into *this; when the int argument is nonzero calls FUN_012ea300(this). |
| `0127c640` | `Engine::LobbyAddLobbyParticipantDisplayAction::DebugName` | ✅ | virtual | Returns the constant C string kAddLobbyParticipantDisplayAction : 0 : ... lobbyhandlers.cc:212 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of its vtable. |
| `0127c650` | `Engine::LobbyAddLobbyParticipantDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `0127f730` | `— ⚠reject` | ⚪ | virtual | Virtual slot [2]. Calls FUN_012e0ba0 and returns 1 when FUN_0127a580() is null. Otherwise reads ValueData entries 0 and 1 as integers, calls FUN_0127f010 with the values and FUN_012d08f0(); calls FUN_0127ede0 when FUN_0… |
