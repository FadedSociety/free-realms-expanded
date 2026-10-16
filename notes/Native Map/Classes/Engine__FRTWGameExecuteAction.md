# Class: Engine::FRTWGameExecuteAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01255d20` | `Engine::FRTWGameExecuteAction::FRTWGameExecuteAction` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWGameExecuteAction::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01255d90` | `Engine::FRTWGameExecuteAction::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name ExecuteAction; no arguments or state. |
| `01255db0` | `Engine::FRTWGameExecuteAction::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the ExecuteAction registration tag (name, version 0, source file and line); no arguments or state. |
| `01255dc0` | `Engine::FRTWGameExecuteAction::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
