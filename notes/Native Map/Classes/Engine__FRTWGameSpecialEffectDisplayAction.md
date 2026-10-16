# Class: Engine::FRTWGameSpecialEffectDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01254320` | `— ⚠reject` | ⚪ | ctor | Constructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then Engine::FRTWGameSpecialEffectDisplayAction::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea300(this). |
| `012543b0` | `Engine::FRTWGameSpecialEffectDisplayAction::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the kSpecialEffectDisplayAction registration tag; no arguments or state. |
| `012543c0` | `Engine::FRTWGameSpecialEffectDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
