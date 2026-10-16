# Class: Engine::FRTWIsPlayerImageLoaded

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01257960` | `— ⚠reject` | ⚪ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWIsPlayerImageLoaded::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `012579d0` | `Engine::FRTWIsPlayerImageLoaded::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name IsPlayerImageLoaded; no arguments or state. |
| `012579f0` | `Engine::FRTWIsPlayerImageLoaded::GetDebugName ⚠downgrade` | 🟢 | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the IsPlayerImageLoaded registration tag (name, version 0, source file and line); no arguments or state. |
| `01257a00` | `Engine::FRTWIsPlayerImageLoaded::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
| `01260bd0` | `— ⚠reject` | ⚪ | virtual | Reads the string argument CharacterID, converts it with __strtoui64 base 10, and when the id is nonzero and the manager at thread-owned object +0xc0 is non-null, calls its vtable +0x3c with the id; when that returns tru… |
