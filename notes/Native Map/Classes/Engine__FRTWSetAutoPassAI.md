# Class: Engine::FRTWSetAutoPassAI

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01252460` | `Engine::FRTWSetAutoPassAI::ctor` | 🟢 | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWSetAutoPassAI::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `012524d0` | `Engine::FRTWSetAutoPassAI::TypeName ⚠downgrade` | 🟢 | virtual | Slot [1] of Engine::FRTWSetAutoPassAI::vftable. Bytes b8 imm32 c3: returns the constant C string 'SetAutoPassAI'. |
| `012524f0` | `Engine::FRTWSetAutoPassAI::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'SetAutoPassAI : 0 : ..\engine\game\frtwgamehandlers.cc:1988'. |
| `01252500` | `Engine::FRTWSetAutoPassAI::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
