# Class: Engine::AccountUsernameRegistrationResultsDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0128e910` | `— ⚠reject` | ⚪ | ctor | Stores Engine::IDisplayActionHandler::vftable then Engine::AccountUsernameRegistrationResultsDisplayAction::vftable into *this with EH state 0; when the int argument is nonzero calls FUN_012ea300(this). |
| `0128e9a0` | `Engine::AccountUsernameRegistrationResultsDisplayAction::GetSourceTag` | ✅ | virtual | Returns the address of the constant string kUsernameRegistrationResultsDisplayAction : 0 : (accounthandlers.cc line 888); 6 bytes, mov eax,imm32 then ret. |
| `0128e9b0` | `Engine::AccountUsernameRegistrationResultsDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |
| `01294de0` | `Engine::AccountUsernameRegistrationResultsDisplayAction::ApplyRegistrationResult` | 🟢 | virtual | Returns 1. Reads arg 0 as a boolean and arg 1 as an integer through FUN_012f8f60, defaulting mismatched types to false and zero. Maps 0x11 to registrationFailureError, 0x4c to loginUsernameRegistrationNeededError, 0x4d … |
