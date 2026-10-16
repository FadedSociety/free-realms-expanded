# Class: Engine::FRTWGameSpellCardsPopulated

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01255e80` | `Engine::FRTWGameSpellCardsPopulated::FRTWGameSpellCardsPopulated` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWGameSpellCardsPopulated::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01255ef0` | `Engine::FRTWGameSpellCardsPopulated::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name SpellCardsPopulated; no arguments or state. |
| `01255f10` | `Engine::FRTWGameSpellCardsPopulated::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the SpellCardsPopulated registration tag (name, version 0, source file and line); no arguments or state. |
| `01255f20` | `Engine::FRTWGameSpellCardsPopulated::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
