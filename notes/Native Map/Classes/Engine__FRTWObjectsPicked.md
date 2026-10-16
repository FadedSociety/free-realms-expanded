# Class: Engine::FRTWObjectsPicked

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01256130` | `Engine::FRTWObjectsPicked::FRTWObjectsPicked` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWObjectsPicked::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `012561a0` | `Engine::FRTWObjectsPicked::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name ObjectsPicked; no arguments or state. |
| `012561c0` | `Engine::FRTWObjectsPicked::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the ObjectsPicked registration tag (name, version 0, source file and line); no arguments or state. |
| `012561d0` | `Engine::FRTWObjectsPicked::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
