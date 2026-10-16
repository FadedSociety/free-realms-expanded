# Class: Engine::IntroduceAccountDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0128d470` | `— ⚠reject` | ⚪ | handler | Type-checks argument 0 as an integer, loads its integer payload from the ValueData object at +8, and when the payload equals FUN_013a3480() runs FUN_013baee0, copies two global strings into local objects, then invokes F… |
| `0128f630` | `Engine::IntroduceAccountDisplayAction::Ctor` | ✅ | ctor | Stores Engine::IDisplayActionHandler::vftable then Engine::IntroduceAccountDisplayAction::vftable into *this with EH state 0; when the int argument is nonzero calls FUN_012ea300(this). |
| `0128f6c0` | `Engine::IntroduceAccountDisplayAction::GetSourceTag` | ✅ | virtual | Returns the address of the constant string kIntroduceAccountDisplayAction : 0 : (accounthandlers.cc line 44); 6 bytes, mov eax,imm32 then ret. |
| `0128f6d0` | `Engine::IntroduceAccountDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |
