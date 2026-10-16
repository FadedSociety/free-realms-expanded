# Class: Engine::FRTWLobbyCancelQueryMatch

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012796b0` | `Engine::FRTWLobbyCancelQueryMatch::ctor` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWLobbyCancelQueryMatch::vftable into *this; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01279720` | `Engine::FRTWLobbyCancelQueryMatch::ActionName` | ✅ | virtual | Returns the constant C string CancelQueryMatch (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::FRTWLobbyCancelQueryMatch::vftable. |
| `01279740` | `Engine::FRTWLobbyCancelQueryMatch::DebugName` | ✅ | virtual | Returns the constant C string CancelQueryMatch : 0 : .\engine\lobby\frtwlobbyhandlers.cc:120 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::FRTWLobbyCancelQueryMatch::vftable. |
| `01279750` | `Engine::FRTWLobbyCancelQueryMatch::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `0127a880` | `Engine::FRTWLobbyCancelQueryMatch::Handle` | ✅ | virtual | Virtual slot [2] of Engine::FRTWLobbyCancelQueryMatch::vftable (434 bytes). Reads the GroupID value, builds a CancelCreateMatch command through FUN_012ec110 and posts it, then builds a GoTo command with the Guard field … |
