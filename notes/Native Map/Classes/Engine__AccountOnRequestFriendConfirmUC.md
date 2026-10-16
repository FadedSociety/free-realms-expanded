# Class: Engine::AccountOnRequestFriendConfirmUC

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0128df80` | `Engine::AccountOnRequestFriendConfirmUC::Ctor` | ✅ | ctor | Stores Engine::IDisplayActionHandler::vftable then Engine::AccountOnRequestFriendConfirmUC::vftable into *this with EH state 0; when the int argument is nonzero calls FUN_012ea300(this). |
| `0128e010` | `Engine::AccountOnRequestFriendConfirmUC::GetSourceTag` | ✅ | virtual | Returns the address of the constant string kOnRequestFriendConfirmUC : 0 : (accounthandlers.cc line 496); 6 bytes, mov eax,imm32 then ret. |
| `0128e020` | `Engine::AccountOnRequestFriendConfirmUC::ScalarDeletingDtor` | ✅ | dtor | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |
| `012928f0` | `Engine::AccountOnRequestFriendConfirmUC::ApplyFriendConfirmResult` | 🟢 | virtual | Returns 1. Reads args 0,1,3,4 as strings, arg 2 as an integer, and arg 5 as a boolean through FUN_012f8f60. When arg 2 is zero, arg 5 false bypasses the success handling; arg 5 true calls FUN_012cfde0 and FUN_012d00f0 w… |
