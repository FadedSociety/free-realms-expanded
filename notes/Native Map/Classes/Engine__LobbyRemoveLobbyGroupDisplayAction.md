# Class: Engine::LobbyRemoveLobbyGroupDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127c1c0` | `Engine::LobbyRemoveLobbyGroupDisplayAction::ctor` | ✅ | ctor | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::LobbyRemoveLobbyGroupDisplayAction::vftable into *this; when the int argument is nonzero calls FUN_012ea300(this). |
| `0127c250` | `Engine::LobbyRemoveLobbyGroupDisplayAction::DebugName` | ✅ | virtual | Returns the constant C string kRemoveLobbyGroupDisplayAction : 0 : ... lobbyhandlers.cc:141 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of its vtable. |
| `0127c260` | `Engine::LobbyRemoveLobbyGroupDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `0127f340` | `Engine::LobbyRemoveLobbyGroupDisplayAction::Vslot2` | 🟢 | virtual | Virtual slot [2]. Calls FUN_012e0ba0(), returns 1 when FUN_0127a580() is null. Otherwise reads ValueData entry 0 as an integer (type check against kIntegerTypeID; 0 on failure), tests it with FUN_013b1580, and when true… |
