# Class: Engine::FRTWUILobbyStartSinglePlayerGame

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01279bf0` | `Engine::FRTWUILobbyStartSinglePlayerGame::ctor` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWUILobbyStartSinglePlayerGame::vftable into *this; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01279c60` | `Engine::FRTWUILobbyStartSinglePlayerGame::ActionName` | ✅ | virtual | Returns the constant C string StartSinglePlayerGame (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::FRTWUILobbyStartSinglePlayerGame::vftable. |
| `01279c80` | `Engine::FRTWUILobbyStartSinglePlayerGame::DebugName` | ✅ | virtual | Returns the constant C string StartSinglePlayerGame : 0 : .\engine\lobby\frtwlobbyhandlers.cc:354 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::FRTWUILobbyStartSinglePlayerGame::vftable. |
| `01279c90` | `Engine::FRTWUILobbyStartSinglePlayerGame::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `0127aa40` | `Engine::FRTWUILobbyStartSinglePlayerGame::Handle ⚠downgrade` | 🟢 | virtual | Virtual slot [2] of Engine::FRTWUILobbyStartSinglePlayerGame::vftable (171 bytes). Calls FUN_012e0ba0 and FUN_00f9c130; when the UI object exists calls FUN_01239bc0 with the string UI::GameScreen, -5 and 0; returns 0. |
