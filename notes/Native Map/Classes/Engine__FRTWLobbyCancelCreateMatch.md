# Class: Engine::FRTWLobbyCancelCreateMatch

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01279800` | `Engine::FRTWLobbyCancelCreateMatch::ctor` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWLobbyCancelCreateMatch::vftable into *this; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01279870` | `Engine::FRTWLobbyCancelCreateMatch::ActionName` | ✅ | virtual | Returns the constant C string CancelCreateMatch (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::FRTWLobbyCancelCreateMatch::vftable. |
| `01279890` | `Engine::FRTWLobbyCancelCreateMatch::DebugName` | ✅ | virtual | Returns the constant C string CancelCreateMatch : 0 : .\engine\lobby\frtwlobbyhandlers.cc:134 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::FRTWLobbyCancelCreateMatch::vftable. |
| `012798a0` | `Engine::FRTWLobbyCancelCreateMatch::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `0127a610` | `Engine::FRTWLobbyCancelCreateMatch::Handle` | ✅ | virtual | Virtual slot [2] of Engine::FRTWLobbyCancelCreateMatch::vftable (33 bytes). Calls FUN_012e0ba0, then ThreadOwnedLookup_0127a580; when that returns non-null calls its vtable slot +0x1c; returns 1. |
