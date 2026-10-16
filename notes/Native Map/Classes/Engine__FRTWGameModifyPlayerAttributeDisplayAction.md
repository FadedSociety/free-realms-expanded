# Class: Engine::FRTWGameModifyPlayerAttributeDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012535d0` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWGameModifyPlayerAttributeDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `01253660` | `— ⚠reject` | ⚪ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kModifyPlayerAttributeDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:2425'. |
| `01253670` | `Engine::FRTWGameModifyPlayerAttributeDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
