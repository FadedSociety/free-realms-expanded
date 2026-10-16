# Class: Engine::FRTWGameUnpause

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01255910` | `Engine::FRTWGameUnpause::FRTWGameUnpause` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWGameUnpause::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01255980` | `Engine::FRTWGameUnpause::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name Unpause; no arguments or state. |
| `012559a0` | `Engine::FRTWGameUnpause::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the Unpause registration tag (name, version 0, source file and line); no arguments or state. |
| `012559b0` | `Engine::FRTWGameUnpause::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
| `01255a10` | `Engine::FRTWGameUnpause::VTableSlot2` | 🟢 | virtual | Vtable slot [2] body: calls FUN_012e4830() with no arguments, passes its eax result as ECX to FUN_012ee180(), then returns 1 (ret 8). |
