# Class: Engine::FRTWGameCloseInfo

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01255670` | `Engine::FRTWGameCloseInfo::FRTWGameCloseInfo` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWGameCloseInfo::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `012556e0` | `Engine::FRTWGameCloseInfo::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name CloseInfo; no arguments or state. |
| `01255700` | `Engine::FRTWGameCloseInfo::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the CloseInfo registration tag (name, version 0, source file and line); no arguments or state. |
| `01255710` | `Engine::FRTWGameCloseInfo::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
| `0125fd00` | `Engine::FRTWGameCloseInfo::Vslot2` | ✅ | virtual | Creates an Invoke ScaleformAction whose MethodName is CloseRichInfoDialog (012ec220), queues it (012e5730, 012e5820) and returns 1 (ret 8). |
