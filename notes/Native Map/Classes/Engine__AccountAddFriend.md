# Class: Engine::AccountAddFriend

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0128ebb0` | `— ⚠reject` | ⚪ | ctor | Stores Scaleform::IScaleformActionHandler::vftable then Engine::AccountAddFriend::vftable into *this with EH state 0; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `0128ec20` | `Engine::AccountAddFriend::GetActionName` | ✅ | virtual | Returns the address of the constant string AddFriend (6 bytes, mov eax,imm32 then ret). |
| `0128ec40` | `Engine::AccountAddFriend::GetSourceTag` | ✅ | virtual | Returns the address of the constant string AddFriend : 0 : (accounthandlers.cc line 948); 6 bytes, mov eax,imm32 then ret. |
| `0128ec50` | `Engine::AccountAddFriend::ScalarDeletingDtor` | ✅ | dtor | Stores Scaleform::IScaleformActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |
| `01295220` | `Engine::AccountAddFriend::AddFriendOrShowError` | 🟢 | virtual | Returns 1. Reads AccountID with default zero, passes it to FUN_012dfbc0 and FUN_012de760, and returns early if either FUN_012de760 or FUN_0127ebb0 is zero. Otherwise obtains two name strings from FUN_013a3a20 and FUN_01… |
