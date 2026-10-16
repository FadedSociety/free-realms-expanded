# Class: Engine::AccountShowFriends

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0128ea60` | `Engine::AccountShowFriends::Ctor` | ✅ | ctor | Stores Scaleform::IScaleformActionHandler::vftable then Engine::AccountShowFriends::vftable into *this with EH state 0; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `0128ead0` | `Engine::AccountShowFriends::GetActionName` | ✅ | virtual | Returns the address of the constant string ShowFriends (6 bytes, mov eax,imm32 then ret). |
| `0128eaf0` | `Engine::AccountShowFriends::GetSourceTag` | ✅ | virtual | Returns the address of the constant string ShowFriends : 0 : (accounthandlers.cc line 927); 6 bytes, mov eax,imm32 then ret. |
| `0128eb00` | `Engine::AccountShowFriends::ScalarDeletingDtor` | ✅ | dtor | Stores Scaleform::IScaleformActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |
| `01295090` | `Engine::AccountShowFriends::OpenFriendsListDialog` | ✅ | virtual | Returns 1. If FUN_0127a580 and then FUN_013b15e0 are nonzero, creates OpenDialog, sets GroupID to 2 and its XML value from FUN_013b1200, calls FUN_013afff0 before reading each of PlayerPage and PlayerMaxPage, then sets … |
