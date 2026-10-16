# Class: Engine::FRTWGameSetupDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01253c90` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWGameSetupDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `01253d20` | `Engine::FRTWGameSetupDisplayAction::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kSetupDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:2466'. |
| `01253d30` | `Engine::FRTWGameSetupDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `0125ef40` | `Engine::FRTWGameSetupDisplayAction::Vslot2` | ✅ | virtual | Checks ValueData argument 1 as a boolean (kBooleanTypeID assert); when the argument is not a true boolean it queues an Invoke ScaleformAction with MethodName CloseLoadScreen (012ec220, 012e5730, 012e5820) and touches a … |
