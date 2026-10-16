# Class: Engine::FRTWGameRequestInfo

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01255520` | `Engine::FRTWGameRequestInfo::FRTWGameRequestInfo` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWGameRequestInfo::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01255590` | `Engine::FRTWGameRequestInfo::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name RequestInfo; no arguments or state. |
| `012555b0` | `Engine::FRTWGameRequestInfo::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the RequestInfo registration tag (name, version 0, source file and line); no arguments or state. |
| `012555c0` | `Engine::FRTWGameRequestInfo::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
| `0125f700` | `Engine::FRTWGameRequestInfo::Vslot2` | ✅ | virtual | Reads the int argument ElementID (012eb290) and looks up a card with 0131bd10; when found and 013809a0 returns 0x1ae03 or 0x1ae04 it queues an Invoke ScaleformAction with MethodName RichInfoDialog (012ec220) with Title … |
