# Class: Engine::FRTWGameShuffleBasesDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012541d0` | `— ⚠reject` | ⚪ | ctor | Constructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then Engine::FRTWGameShuffleBasesDisplayAction::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea300(this). |
| `01254260` | `Engine::FRTWGameShuffleBasesDisplayAction::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the kShuffleBasesDisplayAction registration tag; no arguments or state. |
| `01254270` | `Engine::FRTWGameShuffleBasesDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
