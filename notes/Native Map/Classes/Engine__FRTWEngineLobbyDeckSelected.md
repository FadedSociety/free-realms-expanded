# Class: Engine::FRTWEngineLobbyDeckSelected

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01279e90` | `Engine::FRTWEngineLobbyDeckSelected::ctor` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWEngineLobbyDeckSelected::vftable into *this; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01279f00` | `Engine::FRTWEngineLobbyDeckSelected::ActionName` | ✅ | virtual | Returns the constant C string DeckSelected (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::FRTWEngineLobbyDeckSelected::vftable. |
| `01279f20` | `Engine::FRTWEngineLobbyDeckSelected::DebugName` | ✅ | virtual | Returns the constant C string DeckSelected : 1 : .\engine\lobby\frtwlobbyhandlers.cc:378 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::FRTWEngineLobbyDeckSelected::vftable. |
| `01279f30` | `Engine::FRTWEngineLobbyDeckSelected::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `0127aaf0` | `— ⚠reject` | ⚪ | virtual | Virtual slot [2] of Engine::FRTWEngineLobbyDeckSelected::vftable (335 bytes). Reads a string through FUN_012eb250 and the MatchID value through FUN_012eb290; looks up a match object with FUN_012aaeb0; when it exists and… |
