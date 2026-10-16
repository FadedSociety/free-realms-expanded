# Class: Engine::FRTWEngineLobbyStartSinglePlayerGame

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01279d40` | `Engine::FRTWEngineLobbyStartSinglePlayerGame::ctor` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWEngineLobbyStartSinglePlayerGame::vftable into *this; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01279db0` | `Engine::FRTWEngineLobbyStartSinglePlayerGame::ActionName` | ✅ | virtual | Returns the constant C string StartCampaignGame (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::FRTWEngineLobbyStartSinglePlayerGame::vftable. |
| `01279dd0` | `Engine::FRTWEngineLobbyStartSinglePlayerGame::DebugName` | ✅ | virtual | Returns the constant C string StartCampaignGame : 0 : .\engine\lobby\frtwlobbyhandlers.cc:367 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::FRTWEngineLobbyStartSinglePlayerGame::vftable. |
| `01279de0` | `Engine::FRTWEngineLobbyStartSinglePlayerGame::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `0127a640` | `Engine::FRTWEngineLobbyStartSinglePlayerGame::Handle` | ✅ | virtual | Virtual slot [2] of Engine::FRTWEngineLobbyStartSinglePlayerGame::vftable (33 bytes). Calls FUN_012e0ba0, then ThreadOwnedLookup_0127a580; when that returns non-null calls its vtable slot +0x20; returns 1. |
