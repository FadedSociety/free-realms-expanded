# Class: Engine::FRTWClearWaitingDialog

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01251b50` | `Engine::FRTWClearWaitingDialog::ctor` | 🟢 | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWClearWaitingDialog::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `01251be0` | `Engine::FRTWClearWaitingDialog::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kClearWaitingDialogDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:352'. |
| `01251bf0` | `Engine::FRTWClearWaitingDialog::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
