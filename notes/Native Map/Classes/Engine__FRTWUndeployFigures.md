# Class: Engine::FRTWUndeployFigures

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01252310` | `Engine::FRTWUndeployFigures::ctor` | 🟢 | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWUndeployFigures::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01252380` | `Engine::FRTWUndeployFigures::TypeName ⚠downgrade` | 🟢 | virtual | Slot [1] of Engine::FRTWUndeployFigures::vftable. Bytes b8 imm32 c3: returns the constant C string 'UndeployFigures'. |
| `012523a0` | `Engine::FRTWUndeployFigures::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'UndeployFigures : 0 : ..\engine\game\frtwgamehandlers.cc:1980'. |
| `012523b0` | `Engine::FRTWUndeployFigures::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
