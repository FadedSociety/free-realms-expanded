# Class: Engine::AccountAddIgnore

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0128ee50` | `— ⚠reject` | ⚪ | ctor | Stores Scaleform::IScaleformActionHandler::vftable then Engine::AccountAddIgnore::vftable into *this with EH state 0; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `0128eec0` | `Engine::AccountAddIgnore::GetActionName` | ✅ | virtual | Returns the address of the constant string AddIgnore (6 bytes, mov eax,imm32 then ret). |
| `0128eee0` | `Engine::AccountAddIgnore::GetSourceTag` | ✅ | virtual | Returns the address of the constant string AddIgnore : 0 : (accounthandlers.cc line 1024); 6 bytes, mov eax,imm32 then ret. |
| `0128eef0` | `Engine::AccountAddIgnore::ScalarDeletingDtor` | ✅ | dtor | Stores Scaleform::IScaleformActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |
| `01295650` | `Engine::AccountAddIgnore::AddIgnoreOrShowError` | 🟢 | virtual | Returns 1. Reads AccountID with default zero, passes it to FUN_012dfbc0 and FUN_012de760, and proceeds only when FUN_012de760 and FUN_0127ebb0 are both nonzero. It obtains two name strings from FUN_013a3a20 and FUN_013a… |
