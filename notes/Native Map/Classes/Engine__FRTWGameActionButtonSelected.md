# Class: Engine::FRTWGameActionButtonSelected

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01255bd0` | `Engine::FRTWGameActionButtonSelected::FRTWGameActionButtonSelected` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWGameActionButtonSelected::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01255c40` | `Engine::FRTWGameActionButtonSelected::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name ActionButtonSelected; no arguments or state. |
| `01255c60` | `Engine::FRTWGameActionButtonSelected::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the ActionButtonSelected registration tag (name, version 0, source file and line); no arguments or state. |
| `01255c70` | `Engine::FRTWGameActionButtonSelected::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
