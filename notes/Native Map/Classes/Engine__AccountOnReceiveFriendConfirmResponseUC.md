# Class: Engine::AccountOnReceiveFriendConfirmResponseUC

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0128d8f0` | `Engine::AccountOnReceiveFriendConfirmResponseUC::Ctor` | ✅ | ctor | Stores Engine::IDisplayActionHandler::vftable then Engine::AccountOnReceiveFriendConfirmResponseUC::vftable into *this with EH state 0; when the int argument is nonzero calls FUN_012ea300(this). |
| `0128d980` | `Engine::AccountOnReceiveFriendConfirmResponseUC::GetSourceTag` | ✅ | virtual | Returns the address of the constant string kOnReceiveFriendConfirmResponseUC : 0 : (accounthandlers.cc line 132); 6 bytes, mov eax,imm32 then ret. |
| `0128d990` | `Engine::AccountOnReceiveFriendConfirmResponseUC::ScalarDeletingDtor` | ✅ | dtor | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |
| `01290210` | `— ⚠reject` | ⚪ | handler | Type-checks string arguments 0 and 1, integer argument 2 and boolean argument 3; on the decline result opens a GenericInfoDialog titled Friend Declined with the text formatted from %s has declined your request to be a f… |
