# Class: Engine::FRTWGameSetPhaseDialogDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01253de0` | `Engine::FRTWGameSetPhaseDialogDisplayAction::ctor` | 🟢 | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWGameSetPhaseDialogDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `01253e70` | `Engine::FRTWGameSetPhaseDialogDisplayAction::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kSetPhaseDialogDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:2480'. |
| `01253e80` | `Engine::FRTWGameSetPhaseDialogDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `0125f040` | `— ⚠reject` | ⚪ | virtual | Reads string arguments 1 to 5 (kStringTypeID asserts) and asserts integer arguments 6 and 7; the value of argument 7 is read. When argument 1 equals CustomChoice (00f942a0), argument 7 equals the thread-owned object's f… |
