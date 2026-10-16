# Class: Engine::FRTWGameTieDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01253f30` | `— ⚠reject` | ⚪ | ctor | Constructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then Engine::FRTWGameTieDisplayAction::vftable (EH frame set up first); when the int argument is nonzero calls FUN_012ea300(this). |
| `01253fc0` | `Engine::FRTWGameTieDisplayAction::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) holding the kTieDisplayAction registration tag with source file and line; no arguments or state. |
| `01253fd0` | `Engine::FRTWGameTieDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
| `0125f620` | `Engine::FRTWGameTieDisplayAction::Vslot2` | ✅ | virtual | Gets the thread-owned object (0124d010) and, when non-null, calls 0131de20 and 0131de60; queues an Invoke ScaleformAction with MethodName DoTie (012ec220, 012e5730, 012e5820) and calls 012e4830 and 012eeb40; returns 1. |
