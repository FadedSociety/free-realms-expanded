# Class: Engine::FRTWEngineDeathDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01254f50` | `Engine::FRTWEngineDeathDisplayAction::FRTWEngineDeathDisplayAction` | ✅ | ctor | Constructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then Engine::FRTWEngineDeathDisplayAction::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea300(this). |
| `01254fe0` | `Engine::FRTWEngineDeathDisplayAction::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the kDeathDisplayAction registration tag; no arguments or state. |
| `01254ff0` | `Engine::FRTWEngineDeathDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
| `01255050` | `Engine::FRTWEngineDeathDisplayAction::VTableSlot2` | 🟢 | virtual | Vtable slot [2] body: calls FUN_012e4830() with no arguments, passes its eax result as ECX to FUN_012eeb40(), then returns 0 (ret 8). |
