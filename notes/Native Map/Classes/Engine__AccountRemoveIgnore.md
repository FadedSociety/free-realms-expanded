# Class: Engine::AccountRemoveIgnore

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0128efa0` | `— ⚠reject` | ⚪ | ctor | Stores Scaleform::IScaleformActionHandler::vftable then Engine::AccountRemoveIgnore::vftable into *this with EH state 0; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `0128f010` | `Engine::AccountRemoveIgnore::GetActionName` | ✅ | virtual | Returns the address of the constant string RemoveIgnore (6 bytes, mov eax,imm32 then ret). |
| `0128f030` | `Engine::AccountRemoveIgnore::GetSourceTag` | ✅ | virtual | Returns the address of the constant string RemoveIgnore : 0 : (accounthandlers.cc line 1058); 6 bytes, mov eax,imm32 then ret. |
| `0128f040` | `Engine::AccountRemoveIgnore::ScalarDeletingDtor` | ✅ | dtor | Stores Scaleform::IScaleformActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |
| `0128f980` | `Engine::AccountRemoveIgnore::Handle` | 🟢 | handler | Reads and converts the AccountID integer argument, then proceeds only when that conversion is nonzero and FUN_0127ebb0() succeeds; copies the two strings returned by FUN_013a3a20() and FUN_013a3a30() and passes those st… |
