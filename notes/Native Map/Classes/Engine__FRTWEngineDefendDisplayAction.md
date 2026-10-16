# Class: Engine::FRTWEngineDefendDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01254de0` | `Engine::FRTWEngineDefendDisplayAction::FRTWEngineDefendDisplayAction` | ✅ | ctor | Constructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then Engine::FRTWEngineDefendDisplayAction::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea300(this). |
| `01254e70` | `Engine::FRTWEngineDefendDisplayAction::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the kDefendDisplayAction registration tag; no arguments or state. |
| `01254e80` | `Engine::FRTWEngineDefendDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
| `01254ee0` | `— ⚠reject` | ⚪ | virtual | Vtable slot [2] body: pushes the float at global DAT_017ea718 as the argument of FUN_012e4830, passes its eax result as ECX to FUN_012eeb70(), then returns 0 (ret 8). |
