# Class: Engine::FRTWGameRebuildInitiativeListDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01253b30` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWGameRebuildInitiativeListDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `01253bc0` | `— ⚠reject` | ⚪ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kRebuildInitiativeListDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:2460'. |
| `01253bd0` | `Engine::FRTWGameRebuildInitiativeListDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
