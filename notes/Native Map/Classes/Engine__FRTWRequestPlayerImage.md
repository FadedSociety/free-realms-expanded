# Class: Engine::FRTWRequestPlayerImage

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01257810` | `— ⚠reject` | ⚪ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWRequestPlayerImage::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01257880` | `Engine::FRTWRequestPlayerImage::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name RequestPlayerImage; no arguments or state. |
| `012578a0` | `Engine::FRTWRequestPlayerImage::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the RequestPlayerImage registration tag (name, version 0, source file and line); no arguments or state. |
| `012578b0` | `Engine::FRTWRequestPlayerImage::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
| `01260a50` | `— ⚠reject` | ⚪ | virtual | Reads the string argument CharacterID (012eb250), converts it with __strtoui64 base 10, and when the id is nonzero and the thread-owned object (0124d010) has a manager at +0xc0: if the manager's vtable +0x3c returns fal… |
