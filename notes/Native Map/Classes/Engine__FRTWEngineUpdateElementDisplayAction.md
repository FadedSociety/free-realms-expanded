# Class: Engine::FRTWEngineUpdateElementDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01252700` | `Engine::FRTWEngineUpdateElementDisplayAction::ctor` | 🟢 | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWEngineUpdateElementDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `01252790` | `Engine::FRTWEngineUpdateElementDisplayAction::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kUpdateElementDisplayAction : 1 : ..\engine\game\frtwgamehandlers.cc:2004'. |
| `012527a0` | `Engine::FRTWEngineUpdateElementDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `0125e100` | `— ⚠reject` | ⚪ | virtual | Reads ValueData arguments 1 and 2 as integers (kIntegerTypeID assert) and argument 3 as a value entry; after a 0131bd10 lookup it queues an Invoke ScaleformAction with MethodName UpdateElement and ElementID (012ec220, 0… |
