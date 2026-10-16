# Class: Engine::LobbyLaunchGameDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127d060` | `Engine::LobbyLaunchGameDisplayAction::ctor` | ✅ | ctor | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::LobbyLaunchGameDisplayAction::vftable into *this; when the int argument is nonzero calls FUN_012ea300(this). |
| `0127d0f0` | `Engine::LobbyLaunchGameDisplayAction::DebugName` | ✅ | virtual | Returns the constant C string kLaunchGameDisplayAction : 0 : ... lobbyhandlers.cc:348 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of its vtable. |
| `0127d100` | `Engine::LobbyLaunchGameDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `0127ffc0` | `Engine::LobbyLaunchGameDisplayAction::Vslot2` | 🟢 | virtual | Virtual slot [2]. When FUN_00f9c130() is nonzero reads ValueData entry 0 as an integer (assert and 0 on failure), then calls FUN_01239bc0 with the tag UI::GameScreen, that value and 0. Returns 1. |
