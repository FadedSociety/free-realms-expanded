# Class: Engine::AccountRemoveFriend

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0128ed00` | `Engine::AccountRemoveFriend::Ctor` | ✅ | ctor | Stores Scaleform::IScaleformActionHandler::vftable then Engine::AccountRemoveFriend::vftable into *this with EH state 0; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `0128ed70` | `Engine::AccountRemoveFriend::GetActionName` | ✅ | virtual | Returns the address of the constant string RemoveFriend (6 bytes, mov eax,imm32 then ret). |
| `0128ed90` | `Engine::AccountRemoveFriend::GetSourceTag` | ✅ | virtual | Returns the address of the constant string RemoveFriend : 0 : (accounthandlers.cc line 997); 6 bytes, mov eax,imm32 then ret. |
| `0128eda0` | `Engine::AccountRemoveFriend::ScalarDeletingDtor` | ✅ | dtor | Stores Scaleform::IScaleformActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |
| `0128f830` | `Engine::AccountRemoveFriend::Handle` | 🟢 | handler | Reads and converts the AccountID integer argument, then proceeds only when that conversion is nonzero and FUN_0127ebb0() succeeds; copies the two strings returned by FUN_013a3a20() and FUN_013a3a30() and passes those st… |
