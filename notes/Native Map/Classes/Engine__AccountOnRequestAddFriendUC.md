# Class: Engine::AccountOnRequestAddFriendUC

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0128dce0` | `Engine::AccountOnRequestAddFriendUC::Ctor` | ✅ | ctor | Stores Engine::IDisplayActionHandler::vftable then Engine::AccountOnRequestAddFriendUC::vftable into *this with EH state 0; when the int argument is nonzero calls FUN_012ea300(this). |
| `0128dd70` | `Engine::AccountOnRequestAddFriendUC::GetSourceTag` | ✅ | virtual | Returns the address of the constant string kOnRequestAddFriendUC : 0 : (accounthandlers.cc line 283); 6 bytes, mov eax,imm32 then ret. |
| `0128dd80` | `Engine::AccountOnRequestAddFriendUC::ScalarDeletingDtor` | ✅ | dtor | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |
| `01291250` | `Engine::AccountOnRequestAddFriendUC::ApplyAddFriendResult` | 🟢 | virtual | Returns 1. Reads args 0,1,3,4,5 as strings, arg 2 as an integer, and only type-checks arg 6 as boolean through FUN_012f8f60. If FUN_012d08e0 is nonzero, replaces args 0 and 1 with FUN_013a3a20 and FUN_013a3a30 results. … |
