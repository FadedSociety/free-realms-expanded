# Class: Engine::FRTWGameShowEndOfGameDialogDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01253060` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWGameShowEndOfGameDialogDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `012530f0` | `Engine::FRTWGameShowEndOfGameDialogDisplayAction::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kShowEndOfGameDialogDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:2349'. |
| `01253100` | `Engine::FRTWGameShowEndOfGameDialogDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `0125ea20` | `— ⚠reject` | ⚪ | virtual | Reads int argument 1 (kIntegerTypeID assert) and boolean argument 2 (kBooleanTypeID assert). Sets a winner value to 1 when argument 1 equals the result of the thread-owned object's vtable +0x1c call, else 0. Builds Matc… |
