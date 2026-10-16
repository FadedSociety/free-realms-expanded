# Class: Engine::FRTWSelectRandomHandCard

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012549d0` | `— ⚠reject` | ⚪ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWSelectRandomHandCard::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01254a40` | `Engine::FRTWSelectRandomHandCard::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name SelectRandomHandCard; no arguments or state. |
| `01254a60` | `Engine::FRTWSelectRandomHandCard::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the SelectRandomHandCard registration tag (name, version 0, source file and line); no arguments or state. |
| `01254a70` | `Engine::FRTWSelectRandomHandCard::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
