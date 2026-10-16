# Class: Engine::FRTWReadyApplyModifier

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01252070` | `Engine::FRTWReadyApplyModifier::ctor` | 🟢 | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWReadyApplyModifier::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `012520e0` | `Engine::FRTWReadyApplyModifier::TypeName` | ✅ | virtual | Slot [1] of Engine::FRTWReadyApplyModifier::vftable. Bytes b8 imm32 c3: returns the constant C string 'ReadyApplyModifier'. |
| `01252100` | `Engine::FRTWReadyApplyModifier::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'ReadyApplyModifier : 0 : ..\engine\game\frtwgamehandlers.cc:1962'. |
| `01252110` | `Engine::FRTWReadyApplyModifier::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
