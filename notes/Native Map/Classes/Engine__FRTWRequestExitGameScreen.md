# Class: Engine::FRTWRequestExitGameScreen

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012563d0` | `Engine::FRTWRequestExitGameScreen::FRTWRequestExitGameScreen` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWRequestExitGameScreen::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01256440` | `Engine::FRTWRequestExitGameScreen::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name RequestExitGameScreen; no arguments or state. |
| `01256460` | `Engine::FRTWRequestExitGameScreen::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the RequestExitGameScreen registration tag (name, version 0, source file and line); no arguments or state. |
| `01256470` | `Engine::FRTWRequestExitGameScreen::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
| `012601f0` | `Engine::FRTWRequestExitGameScreen::Vslot2` | ✅ | virtual | Calls virtual slot +0x20 (no arguments) on the thread-owned object from 0124d010 when it is non-null, creates a ScaleformAction named ChangeToPreviousScreen (012ec110), queues it (012e5730, 012e5820), and returns 1. |
