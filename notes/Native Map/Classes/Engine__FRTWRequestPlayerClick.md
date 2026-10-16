# Class: Engine::FRTWRequestPlayerClick

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012518b0` | `Engine::FRTWRequestPlayerClick::ctor` | 🟢 | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWRequestPlayerClick::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `01251940` | `Engine::FRTWRequestPlayerClick::DebugName` | ✅ | virtual | Slot [4] of Engine::FRTWRequestPlayerClick::vftable. Bytes b8 imm32 c3: returns the constant C string 'kRequestPlayerClickDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:333'. |
| `01251950` | `Engine::FRTWRequestPlayerClick::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
