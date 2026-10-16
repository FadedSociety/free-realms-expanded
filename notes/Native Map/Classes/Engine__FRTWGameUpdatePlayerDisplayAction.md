# Class: Engine::FRTWGameUpdatePlayerDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01253730` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWGameUpdatePlayerDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `012537c0` | `— ⚠reject` | ⚪ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kUpdatePlayerDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:2433'. |
| `012537d0` | `Engine::FRTWGameUpdatePlayerDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
