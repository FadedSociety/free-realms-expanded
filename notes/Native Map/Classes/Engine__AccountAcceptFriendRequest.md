# Class: Engine::AccountAcceptFriendRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0128f0f0` | `— ⚠reject` | ⚪ | ctor | Stores Scaleform::IScaleformActionHandler::vftable then Engine::AccountAcceptFriendRequest::vftable into *this with EH state 0; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `0128f160` | `Engine::AccountAcceptFriendRequest::GetActionName` | ✅ | virtual | Returns the address of the constant string AcceptFriendRequest (6 bytes, mov eax,imm32 then ret). |
| `0128f180` | `Engine::AccountAcceptFriendRequest::GetSourceTag` | ✅ | virtual | Returns the address of the constant string AcceptFriendRequest : 0 : (accounthandlers.cc line 1086); 6 bytes, mov eax,imm32 then ret. |
| `0128f190` | `Engine::AccountAcceptFriendRequest::ScalarDeletingDtor` | ✅ | dtor | Stores Scaleform::IScaleformActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |
| `0128fad0` | `Engine::AccountAcceptFriendRequest::Handle` | 🟢 | handler | Gates on FUN_0127ebb0 and FUN_013bdf40 (which returns the first element of the vector at +0x20, or 0); builds a local UChatCommand_RequestFriendConfirmReciprocate object with FUN_013c0860, sets its accept byte to 1 thro… |
