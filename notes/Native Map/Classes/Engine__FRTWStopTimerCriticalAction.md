# Class: Engine::FRTWStopTimerCriticalAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01252c60` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWStopTimerCriticalAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `01252cf0` | `Engine::FRTWStopTimerCriticalAction::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kStopTimerCriticalAction : 0 : ..\engine\game\frtwgamehandlers.cc:2292'. |
| `01252d00` | `Engine::FRTWStopTimerCriticalAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `0125e970` | `Engine::FRTWStopTimerCriticalAction::Vslot2` | ✅ | virtual | Creates an Invoke ScaleformAction whose MethodName is EndGameTimer (012ec220), queues it (012e5730, 012e5820) and returns 1 (ret 8). |
