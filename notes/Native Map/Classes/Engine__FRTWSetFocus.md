# Class: Engine::FRTWSetFocus

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01254470` | `— ⚠reject` | ⚪ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWSetFocus::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `012544e0` | `Engine::FRTWSetFocus::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name SetFocus; no arguments or state. |
| `01254500` | `Engine::FRTWSetFocus::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the SetFocus registration tag (name, version 0, source file and line); no arguments or state. |
| `01254510` | `Engine::FRTWSetFocus::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
