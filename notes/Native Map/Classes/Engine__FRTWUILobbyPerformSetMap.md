# Class: Engine::FRTWUILobbyPerformSetMap

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01279fe0` | `Engine::FRTWUILobbyPerformSetMap::ctor` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWUILobbyPerformSetMap::vftable into *this; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `0127a050` | `Engine::FRTWUILobbyPerformSetMap::ActionName` | ✅ | virtual | Returns the constant C string PerformSetMap (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::FRTWUILobbyPerformSetMap::vftable. |
| `0127a070` | `Engine::FRTWUILobbyPerformSetMap::DebugName` | ✅ | virtual | Returns the constant C string PerformSetMap : 0 : .\engine\lobby\frtwlobbyhandlers.cc:400 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::FRTWUILobbyPerformSetMap::vftable. |
| `0127a080` | `Engine::FRTWUILobbyPerformSetMap::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
