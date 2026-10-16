# Class: Engine::FRTWGameUpdateWaitingHintsDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01253470` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWGameUpdateWaitingHintsDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `01253500` | `Engine::FRTWGameUpdateWaitingHintsDisplayAction::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kUpdateWaitingHintsDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:2418'. |
| `01253510` | `Engine::FRTWGameUpdateWaitingHintsDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
