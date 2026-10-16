# Class: Engine::FRTWGameSetActivePlayerDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01253310` | `Engine::FRTWGameSetActivePlayerDisplayAction::ctor` | 🟢 | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWGameSetActivePlayerDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `012533a0` | `— ⚠reject` | ⚪ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kSetActivePlayerDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:2411'. |
| `012533b0` | `Engine::FRTWGameSetActivePlayerDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
