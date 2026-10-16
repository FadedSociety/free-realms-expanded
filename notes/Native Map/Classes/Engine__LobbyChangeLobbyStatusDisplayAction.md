# Class: Engine::LobbyChangeLobbyStatusDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127c850` | `Engine::LobbyChangeLobbyStatusDisplayAction::ctor` | ✅ | ctor | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::LobbyChangeLobbyStatusDisplayAction::vftable into *this; when the int argument is nonzero calls FUN_012ea300(this). |
| `0127c8e0` | `Engine::LobbyChangeLobbyStatusDisplayAction::DebugName` | ✅ | virtual | Returns the constant C string kChangeLobbyStatusDisplayAction : 0 : ... lobbyhandlers.cc:273 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of its vtable. |
| `0127c8f0` | `Engine::LobbyChangeLobbyStatusDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `0127fb10` | `Engine::LobbyChangeLobbyStatusDisplayAction::Vslot2` | 🟢 | virtual | Virtual slot [2]. Returns 1 when FUN_00f9c010() is zero. Otherwise reads ValueData entry 0 as an integer, entry 1 as an integer and entry 2 as a boolean (type 1; assert mTypeID == kBooleanTypeID and 0 on failure). Calls… |
