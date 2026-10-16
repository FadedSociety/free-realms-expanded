# Class: Engine::FRTWGameRotateList

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012557c0` | `Engine::FRTWGameRotateList::FRTWGameRotateList` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWGameRotateList::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01255830` | `Engine::FRTWGameRotateList::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name RotateList; no arguments or state. |
| `01255850` | `Engine::FRTWGameRotateList::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the RotateList registration tag (name, version 0, source file and line); no arguments or state. |
| `01255860` | `Engine::FRTWGameRotateList::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
| `0125fdb0` | `Engine::FRTWGameRotateList::Vslot2` | ✅ | virtual | Reads the int argument Direction (012eb290). With the thread-owned object (0124d010) and a non-empty list at object+0x44/+0x48, finds the index of the current value at object+0x38; it sets object+0x38 to the next list e… |
