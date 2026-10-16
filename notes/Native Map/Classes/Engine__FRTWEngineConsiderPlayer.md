# Class: Engine::FRTWEngineConsiderPlayer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01254b20` | `— ⚠reject` | ⚪ | ctor | Constructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then Engine::FRTWEngineConsiderPlayer::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea300(this). |
| `01254bb0` | `Engine::FRTWEngineConsiderPlayer::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the kConsiderPlayerDisplayAction registration tag; no arguments or state. |
| `01254bc0` | `Engine::FRTWEngineConsiderPlayer::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
