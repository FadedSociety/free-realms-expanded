# Class: Engine::LobbyRemoveLobbyParticipantDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127c310` | `Engine::LobbyRemoveLobbyParticipantDisplayAction::ctor` | ✅ | ctor | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::LobbyRemoveLobbyParticipantDisplayAction::vftable into *this; when the int argument is nonzero calls FUN_012ea300(this). |
| `0127c3a0` | `Engine::LobbyRemoveLobbyParticipantDisplayAction::DebugName` | ✅ | virtual | Returns the constant C string kRemoveLobbyParticipantDisplayAction : 0 : ... lobbyhandlers.cc:159 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of its vtable. |
| `0127c3b0` | `Engine::LobbyRemoveLobbyParticipantDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `0127f3d0` | `Engine::LobbyRemoveLobbyParticipantDisplayAction::Vslot2` | 🟢 | virtual | Virtual slot [2]. Calls FUN_012e0ba0 and returns 1 when FUN_0127a580() is null. Otherwise reads ValueData entries 0 and 1 as integers (type check, 0 on failure), calls FUN_0127ede0 when FUN_013b15f0 is nonzero and FUN_0… |
