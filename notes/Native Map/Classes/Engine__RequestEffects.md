# Class: Engine::RequestEffects

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01256960` | `Engine::RequestEffects::RequestEffects` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::RequestEffects::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `012569d0` | `Engine::RequestEffects::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name RequestEffects; no arguments or state. |
| `012569f0` | `Engine::RequestEffects::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the RequestEffects registration tag (name, version 0, source file and line); no arguments or state. |
| `01256a00` | `Engine::RequestEffects::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
