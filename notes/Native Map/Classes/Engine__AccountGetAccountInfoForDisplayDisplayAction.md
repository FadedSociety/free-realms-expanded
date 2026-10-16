# Class: Engine::AccountGetAccountInfoForDisplayDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0128e770` | `Engine::AccountGetAccountInfoForDisplayDisplayAction::Ctor` | ✅ | ctor | Stores Engine::IDisplayActionHandler::vftable then Engine::AccountGetAccountInfoForDisplayDisplayAction::vftable into *this with EH state 0; when the int argument is nonzero calls FUN_012ea300(this). |
| `0128e800` | `Engine::AccountGetAccountInfoForDisplayDisplayAction::GetSourceTag` | ✅ | virtual | Returns the address of the constant string kGetAccountInfoForDisplayDisplayAction : 0 : (accounthandlers.cc line 852); 6 bytes, mov eax,imm32 then ret. |
| `0128e810` | `Engine::AccountGetAccountInfoForDisplayDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |
| `01294b00` | `Engine::AccountGetAccountInfoForDisplayDisplayAction::OpenPlayerInfoDialog` | ✅ | virtual | Returns 1. Reads arg 0 as an integer through FUN_012f8f60, using zero on a tag mismatch; sets AccountID, XML, HonorsXML, AvatarXML and MatchesXML on an OpenDialog action. The four XML values use DAT_018d3f54, whose byte… |
