# Class: Engine::FRTWSpawnWaitingDialog

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01251a00` | `Engine::FRTWSpawnWaitingDialog::ctor` | 🟢 | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWSpawnWaitingDialog::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `01251a90` | `Engine::FRTWSpawnWaitingDialog::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kSpawnWaitingDialogDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:343'. |
| `01251aa0` | `Engine::FRTWSpawnWaitingDialog::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
