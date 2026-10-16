# Class: Engine::FRTWGameGameTimerComplete

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01255a80` | `— ⚠reject` | ⚪ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWGameGameTimerComplete::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01255af0` | `Engine::FRTWGameGameTimerComplete::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name GameTimerComplete; no arguments or state. |
| `01255b10` | `Engine::FRTWGameGameTimerComplete::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the GameTimerComplete registration tag (name, version 0, source file and line); no arguments or state. |
| `01255b20` | `Engine::FRTWGameGameTimerComplete::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
