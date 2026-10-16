# Class: Engine::FRTWEnginePlayAnimationDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012550c0` | `— ⚠reject` | ⚪ | ctor | Constructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then Engine::FRTWEnginePlayAnimationDisplayAction::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea300(this). |
| `01255150` | `Engine::FRTWEnginePlayAnimationDisplayAction::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the kPlayAnimationDisplayAction registration tag; no arguments or state. |
| `01255160` | `Engine::FRTWEnginePlayAnimationDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
| `012551c0` | `— ⚠reject` | ⚪ | virtual | Vtable slot [2] body: gets DisplayAction argument index 3 via FUN_012f8f60(3), requires its type field at +4 to equal 1 (boolean; otherwise asserts via FUN_012f5a60 and returns 0), and when the flag byte at +8 is nonzer… |
