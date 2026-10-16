# Class: Engine::FRTWConsiderAIResume

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01254730` | `— ⚠reject` | ⚪ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWConsiderAIResume::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `012547a0` | `Engine::FRTWConsiderAIResume::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name ConsiderAIResume; no arguments or state. |
| `012547c0` | `Engine::FRTWConsiderAIResume::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the ConsiderAIResume registration tag (name, version 0, source file and line); no arguments or state. |
| `012547d0` | `Engine::FRTWConsiderAIResume::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
