# Class: Engine::FRTWUnsetAutoPassAI

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012525b0` | `Engine::FRTWUnsetAutoPassAI::ctor` | 🟢 | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWUnsetAutoPassAI::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01252620` | `Engine::FRTWUnsetAutoPassAI::TypeName ⚠downgrade` | 🟢 | virtual | Slot [1] of Engine::FRTWUnsetAutoPassAI::vftable. Bytes b8 imm32 c3: returns the constant C string 'UnsetAutoPassAI'. |
| `01252640` | `Engine::FRTWUnsetAutoPassAI::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'UnsetAutoPassAI : 0 : ..\engine\game\frtwgamehandlers.cc:1996'. |
| `01252650` | `Engine::FRTWUnsetAutoPassAI::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
