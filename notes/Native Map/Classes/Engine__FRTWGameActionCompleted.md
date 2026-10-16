# Class: Engine::FRTWGameActionCompleted

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012553c0` | `Engine::FRTWGameActionCompleted::FRTWGameActionCompleted` | ✅ | ctor | Constructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then Engine::FRTWGameActionCompleted::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea300(this). |
| `01255450` | `Engine::FRTWGameActionCompleted::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the kActionCompletedDisplayAction registration tag; no arguments or state. |
| `01255460` | `Engine::FRTWGameActionCompleted::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
