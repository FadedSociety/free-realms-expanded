# Class: Engine::FRTWLobbyQueryCreateMatch

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127a2b0` | `Engine::FRTWLobbyQueryCreateMatch::ctor` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWLobbyQueryCreateMatch::vftable into *this; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `0127a320` | `Engine::FRTWLobbyQueryCreateMatch::ActionName` | ✅ | virtual | Returns the constant C string QueryCreateMatch (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::FRTWLobbyQueryCreateMatch::vftable. |
| `0127a340` | `Engine::FRTWLobbyQueryCreateMatch::DebugName` | ✅ | virtual | Returns the constant C string QueryCreateMatch : 0 : .\engine\lobby\frtwlobbyhandlers.cc:49 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::FRTWLobbyQueryCreateMatch::vftable. |
| `0127a350` | `Engine::FRTWLobbyQueryCreateMatch::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `0127bb70` | `Engine::FRTWLobbyQueryCreateMatch::Handle` | ✅ | virtual | Virtual slot [2] of Engine::FRTWLobbyQueryCreateMatch::vftable (1423 bytes). Reads the current map entries from a list of *.csv files through FUN_012f2fb0, FUN_01249400 and FUN_012f57f0, builds a Maps list with FileName… |
