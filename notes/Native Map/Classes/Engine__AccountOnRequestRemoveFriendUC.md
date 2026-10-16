# Class: Engine::AccountOnRequestRemoveFriendUC

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0128e370` | `Engine::AccountOnRequestRemoveFriendUC::Ctor` | ✅ | ctor | Stores Engine::IDisplayActionHandler::vftable then Engine::AccountOnRequestRemoveFriendUC::vftable into *this with EH state 0; when the int argument is nonzero calls FUN_012ea300(this). |
| `0128e400` | `Engine::AccountOnRequestRemoveFriendUC::GetSourceTag` | ✅ | virtual | Returns the address of the constant string kOnRequestRemoveFriendUC : 0 : (accounthandlers.cc line 646); 6 bytes, mov eax,imm32 then ret. |
| `0128e410` | `Engine::AccountOnRequestRemoveFriendUC::ScalarDeletingDtor` | ✅ | dtor | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |
| `01293230` | `Engine::AccountOnRequestRemoveFriendUC::ApplyRemoveFriendResult` | 🟢 | virtual | Returns 1. Reads args 0,1,3,4 as strings and arg 2 as an integer through FUN_012f8f60. On code 0, calls FUN_012cfde0 and FUN_012cf2b0 with args 3 and 4; if FUN_0127a580 is nonzero, creates an UpdateFriends action for Fr… |
