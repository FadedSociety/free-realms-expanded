# Class: Engine::AccountSetAchievementListDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0128d640` | `Engine::AccountSetAchievementListDisplayAction::Ctor` | ✅ | ctor | Stores Engine::IDisplayActionHandler::vftable then Engine::AccountSetAchievementListDisplayAction::vftable into *this with EH state 0; when the int argument is nonzero calls FUN_012ea300(this). |
| `0128d6d0` | `Engine::AccountSetAchievementListDisplayAction::GetSourceTag` | ✅ | virtual | Returns the address of the constant string kSetAchievementListDisplayAction : 0 : (accounthandlers.cc line 80); 6 bytes, mov eax,imm32 then ret. |
| `0128d6e0` | `Engine::AccountSetAchievementListDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |
