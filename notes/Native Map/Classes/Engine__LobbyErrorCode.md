# Class: Engine::LobbyErrorCode

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127d1b0` | `Engine::LobbyErrorCode::LobbyErrorCode` | ✅ | ctor | Constructor: stores the base Engine::IDisplayActionHandler vftable into *this, then the Engine::LobbyErrorCode vftable, and returns this. When the second argument is nonzero it also calls FUN_012ea300 on this. |
| `0127d240` | `Engine::LobbyErrorCode::GetLogTag` | ✅ | virtual | Returns a pointer to the literal string kErrorCodeDisplayAction : 1 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:362 (mov eax, imm32; ret). No arguments read. |
| `0127d250` | `Engine::LobbyErrorCode::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores the Engine::IDisplayActionHandler vftable into *this, returns this, and calls _free(this) when bit 0 of the argument is set. |
| `012800b0` | `Engine::LobbyErrorCode::Vslot2` | 🟢 | virtual | Virtual slot [2]. Reads ValueData entry 0 as an integer (assert on a non-integer type, then returns 2). When the value is 0x9e builds the tag matchMakingTimeout with FUN_012ec220 and passes it to FUN_012e5730 and FUN_01… |
