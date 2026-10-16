# Class: Engine::FRTWAddTerrain

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01251ca0` | `Engine::FRTWAddTerrain::ctor` | 🟢 | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWAddTerrain::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `01251d30` | `Engine::FRTWAddTerrain::DebugName` | ✅ | virtual | Slot [4] of Engine::FRTWAddTerrain::vftable. Bytes b8 imm32 c3: returns the constant C string 'kAddTerrainDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:360'. |
| `01251d40` | `Engine::FRTWAddTerrain::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
