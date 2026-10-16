# Class: Engine::LobbyDisplayMOTDDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127cb00` | `Engine::LobbyDisplayMOTDDisplayAction::ctor` | ✅ | ctor | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::LobbyDisplayMOTDDisplayAction::vftable into *this; when the int argument is nonzero calls FUN_012ea300(this). |
| `0127cb90` | `Engine::LobbyDisplayMOTDDisplayAction::DebugName` | ✅ | virtual | Returns the constant C string kDisplayMOTDDisplayAction : 0 : ... lobbyhandlers.cc:303 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of its vtable. |
| `0127cba0` | `Engine::LobbyDisplayMOTDDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `0127fcf0` | `Engine::LobbyDisplayMOTDDisplayAction::Vslot2` | 🟢 | virtual | Virtual slot [2]. Returns when FUN_0127a580() is null. Otherwise reads entry 1 as an integer (assert and 0 on failure), and entry 2 as a C string (type 3; assert mTypeID == kStringTypeID on failure, else null). Passes e… |
