# Class: Engine::AccountUsernameRegistration

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0128f730` | `Engine::AccountUsernameRegistration::Ctor` | ✅ | ctor | Stores Scaleform::IScaleformActionHandler::vftable then Engine::AccountUsernameRegistration::vftable into *this with EH state 0; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `0128f7a0` | `Engine::AccountUsernameRegistration::GetActionName` | ✅ | virtual | Returns the address of the constant string UserNameRegistration (6 bytes, mov eax,imm32 then ret). |
| `0128f7c0` | `Engine::AccountUsernameRegistration::GetSourceTag` | ✅ | virtual | Returns the address of the constant string UserNameRegistration : 0 : (accounthandlers.cc line 873); 6 bytes, mov eax,imm32 then ret. |
| `0128f7d0` | `Engine::AccountUsernameRegistration::ScalarDeletingDtor` | ✅ | dtor | Stores Scaleform::IScaleformActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |
| `01294c60` | `Engine::AccountUsernameRegistration::BuildRegisterUsernameCommand` | 🟢 | virtual | Returns 1. Reads UserName through FUN_012eb250 with the empty-string default and copies it to a local string. Constructs a local LoginCommand_RegisterUsername object (FUN_013c15d0); sets its +4 string to the username, +… |
