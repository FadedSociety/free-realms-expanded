# Class: Engine::FRTWUIUpdateElementDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01252850` | `Engine::FRTWUIUpdateElementDisplayAction::ctor` | 🟢 | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWUIUpdateElementDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `012528e0` | `Engine::FRTWUIUpdateElementDisplayAction::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kUpdateElementDisplayAction : 1 : ..\engine\game\frtwgamehandlers.cc:2136'. |
| `012528f0` | `Engine::FRTWUIUpdateElementDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
