# Class: Engine::FRTWGameUpdateCreatureActionsDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012539e0` | `Engine::FRTWGameUpdateCreatureActionsDisplayAction::ctor` | 🟢 | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWGameUpdateCreatureActionsDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `01253a70` | `— ⚠reject` | ⚪ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kUpdateCreatureActionsDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:2451'. |
| `01253a80` | `Engine::FRTWGameUpdateCreatureActionsDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
