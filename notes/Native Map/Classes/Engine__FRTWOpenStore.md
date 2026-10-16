# Class: Engine::FRTWOpenStore

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01257570` | `— ⚠reject` | ⚪ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then Engine::FRTWOpenStore::vftable (EH frame first); when the int argument is nonzero calls FUN_012ea4f0(this). |
| `012575e0` | `Engine::FRTWOpenStore::GetShortName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the short action name OpenStore; no arguments or state. |
| `01257600` | `Engine::FRTWOpenStore::GetDebugName` | ✅ | virtual | Returns a constant string (6 bytes: mov eax, imm32; ret) with the OpenStore registration tag (name, version 0, source file and line); no arguments or state. |
| `01257610` | `Engine::FRTWOpenStore::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, then when bit 0 of the byte argument is set calls _free(param_1) through thunk 00d82463; returns param_1. |
| `012608e0` | `— ⚠reject` | ⚪ | virtual | Reads int arguments StoreLocation and CategoryID (default -1) and string argument BundleStr; when the thread-owned object (0124d010) has a non-null manager at +0xc0 it calls the manager's virtual slot +0x1c when StoreLo… |
