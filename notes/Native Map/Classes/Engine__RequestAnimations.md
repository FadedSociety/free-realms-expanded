# Class: Engine::RequestAnimations

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01256ec0` | `— ⚠reject` | ⚪ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::RequestAnimations::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01256f30` | `Engine::RequestAnimations::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name RequestAnimations; no arguments or state. |
| `01256f50` | `Engine::RequestAnimations::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the RequestAnimations registration tag (name, version 0, source file and line); no arguments or state. |
| `01256f60` | `Engine::RequestAnimations::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
