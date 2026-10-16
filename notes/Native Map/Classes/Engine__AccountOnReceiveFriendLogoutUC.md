# Class: Engine::AccountOnReceiveFriendLogoutUC

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0128db90` | `Engine::AccountOnReceiveFriendLogoutUC::Ctor` | ✅ | ctor | Stores Engine::IDisplayActionHandler::vftable then Engine::AccountOnReceiveFriendLogoutUC::vftable into *this with EH state 0; when the int argument is nonzero calls FUN_012ea300(this). |
| `0128dc20` | `Engine::AccountOnReceiveFriendLogoutUC::GetSourceTag` | ✅ | virtual | Returns the address of the constant string kOnReceiveFriendLogoutUC : 0 : (accounthandlers.cc line 256); 6 bytes, mov eax,imm32 then ret. |
| `0128dc30` | `Engine::AccountOnReceiveFriendLogoutUC::ScalarDeletingDtor` | ✅ | dtor | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |
| `01290f70` | `Engine::AccountOnReceiveFriendLogoutUC::Handle` | 🟢 | handler | Type-checks string arguments 0 and 1 and integer argument 2, then runs the friend-list refresh: builds an UpdateFriends command (13-character literal) with ListName Friends and a payload from FUN_013b15e0 and FUN_013b12… |
