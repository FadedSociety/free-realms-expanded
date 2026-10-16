# Class: Engine::AccountRejectFriendRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0128f240` | `— ⚠reject` | ⚪ | ctor | Stores Scaleform::IScaleformActionHandler::vftable then Engine::AccountRejectFriendRequest::vftable into *this with EH state 0; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `0128f2b0` | `Engine::AccountRejectFriendRequest::GetActionName` | ✅ | virtual | Returns the address of the constant string RejectFriendRequest (6 bytes, mov eax,imm32 then ret). |
| `0128f2d0` | `Engine::AccountRejectFriendRequest::GetSourceTag` | ✅ | virtual | Returns the address of the constant string RejectFriendRequest : 0 : (accounthandlers.cc line 1127); 6 bytes, mov eax,imm32 then ret. |
| `0128f2e0` | `Engine::AccountRejectFriendRequest::ScalarDeletingDtor` | ✅ | dtor | Stores Scaleform::IScaleformActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |
| `0128fc00` | `Engine::AccountRejectFriendRequest::Handle` | 🟢 | handler | Same body as AcceptFriendRequest::Handle (0128fad0): gates on FUN_0127ebb0 and FUN_013bdf40, builds a UChatCommand_RequestFriendConfirmReciprocate with FUN_013c0860, sets its accept byte to 0 through FUN_013c06a0(0), fi… |
