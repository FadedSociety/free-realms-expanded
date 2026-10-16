# Class: Engine::GetEffectData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01256c00` | `— ⚠reject` | ⚪ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::GetEffectData::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01256c70` | `Engine::GetEffectData::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name GetEffectData; no arguments or state. |
| `01256c90` | `Engine::GetEffectData::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the GetEffectData registration tag (name, version 0, source file and line); no arguments or state. |
| `01256ca0` | `Engine::GetEffectData::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
