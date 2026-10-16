# Class: Engine::FRTWGameResetUIStateDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01252f10` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWGameResetUIStateDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `01252fa0` | `Engine::FRTWGameResetUIStateDisplayAction::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kResetUIStateDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:2331'. |
| `01252fb0` | `Engine::FRTWGameResetUIStateDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
