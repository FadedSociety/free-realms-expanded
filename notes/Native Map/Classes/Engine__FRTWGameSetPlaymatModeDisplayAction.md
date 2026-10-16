# Class: Engine::FRTWGameSetPlaymatModeDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012529b0` | `Engine::FRTWGameSetPlaymatModeDisplayAction::ctor` | 🟢 | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWGameSetPlaymatModeDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `01252a40` | `Engine::FRTWGameSetPlaymatModeDisplayAction::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kSetPlaymatModeDisplayAction : 1 : ..\engine\game\frtwgamehandlers.cc:2233'. |
| `01252a50` | `Engine::FRTWGameSetPlaymatModeDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `0125e700` | `Engine::FRTWGameSetPlaymatModeDisplayAction::Vslot2` | ✅ | virtual | Checks ValueData argument 1 as a string (kStringTypeID assert), then queues an Invoke ScaleformAction with MethodName SetPlaymatMode (012ec220) with the string under the key Mode (012ebe00, 012e5730, 012e5820). When the… |
