# Class: Engine::FRTWGamePhaseTurnChangeDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012531b0` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWGamePhaseTurnChangeDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `01253240` | `Engine::FRTWGamePhaseTurnChangeDisplayAction::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kPhaseTurnChangeDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:2405'. |
| `01253250` | `Engine::FRTWGamePhaseTurnChangeDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
