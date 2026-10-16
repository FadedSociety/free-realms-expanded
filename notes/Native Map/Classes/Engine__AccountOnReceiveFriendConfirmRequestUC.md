# Class: Engine::AccountOnReceiveFriendConfirmRequestUC

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0128d7a0` | `Engine::AccountOnReceiveFriendConfirmRequestUC::Ctor` | ✅ | ctor | Stores Engine::IDisplayActionHandler::vftable then Engine::AccountOnReceiveFriendConfirmRequestUC::vftable into *this with EH state 0; when the int argument is nonzero calls FUN_012ea300(this). |
| `0128d830` | `Engine::AccountOnReceiveFriendConfirmRequestUC::GetSourceTag` | ✅ | virtual | Returns the address of the constant string kOnReceiveFriendConfirmRequestUC : 0 : (accounthandlers.cc line 86); 6 bytes, mov eax,imm32 then ret. |
| `0128d840` | `Engine::AccountOnReceiveFriendConfirmRequestUC::ScalarDeletingDtor` | ✅ | dtor | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |
| `0128fda0` | `— ⚠reject` | ⚪ | handler | Reads string arguments 0 and 1 and integer argument 2 (type-checked), formats a prompt string, and opens a GenericPromptDialog through OpenDialog: TitleText Tcg_dialog.friendInviteTitle, PromptText from the formatted st… |
