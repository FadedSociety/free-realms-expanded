# Class: Engine::FRTWGameQuerySpellInfo

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01255fe0` | `Engine::FRTWGameQuerySpellInfo::FRTWGameQuerySpellInfo` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWGameQuerySpellInfo::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01256050` | `Engine::FRTWGameQuerySpellInfo::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name QuerySpellInfo; no arguments or state. |
| `01256070` | `Engine::FRTWGameQuerySpellInfo::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the QuerySpellInfo registration tag (name, version 1, source file and line); no arguments or state. |
| `01256080` | `Engine::FRTWGameQuerySpellInfo::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
| `0125ffa0` | `— ⚠reject` | ⚪ | virtual | Returns nothing; when the thread-owned object (0124d010) and the object from 0131b3c0 are non-null it reads the int ActionID (012eb290), fills three local strings through 0131edc0, sets the first (Title) to the third st… |
