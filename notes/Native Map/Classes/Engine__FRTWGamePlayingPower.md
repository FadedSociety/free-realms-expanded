# Class: Engine::FRTWGamePlayingPower

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01255260` | `Engine::FRTWGamePlayingPower::FRTWGamePlayingPower` | ✅ | ctor | Constructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then Engine::FRTWGamePlayingPower::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea300(this). |
| `012552f0` | `Engine::FRTWGamePlayingPower::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the kPlayingPowerDisplayAction registration tag; no arguments or state. |
| `01255300` | `Engine::FRTWGamePlayingPower::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
