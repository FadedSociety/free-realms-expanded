# Class: Engine::FRTWGameMoveCardDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01251f20` | `Engine::FRTWGameMoveCardDisplayAction::ctor` | 🟢 | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWGameMoveCardDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `01251fb0` | `Engine::FRTWGameMoveCardDisplayAction::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kMoveCardDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:1860'. |
| `01251fc0` | `Engine::FRTWGameMoveCardDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `0125db40` | `Engine::FRTWGameMoveCardDisplayAction::Vslot2` | ✅ | virtual | Returns 1. Gets the thread-owned object (0124d010), reads ValueData argument 1 as an integer (012f8f60, kIntegerTypeID assert), looks up a model object (0131b390), then queues a MoveModelNodes ScaleformAction (012ec110,… |
