# Class: Engine::FRTWLobbyQueryMatch

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01279560` | `Engine::FRTWLobbyQueryMatch::ctor` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWLobbyQueryMatch::vftable into *this; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `012795d0` | `Engine::FRTWLobbyQueryMatch::ActionName` | ✅ | virtual | Returns the constant C string QueryMatch (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::FRTWLobbyQueryMatch::vftable. |
| `012795f0` | `Engine::FRTWLobbyQueryMatch::DebugName` | ✅ | virtual | Returns the constant C string QueryMatch : 0 : .\engine\lobby\frtwlobbyhandlers.cc:87 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::FRTWLobbyQueryMatch::vftable. |
| `01279600` | `Engine::FRTWLobbyQueryMatch::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `0127a670` | `Engine::FRTWLobbyQueryMatch::Handle` | ✅ | virtual | Virtual slot [2] of Engine::FRTWLobbyQueryMatch::vftable (517 bytes). Reads the GroupID value, looks the lobby object up through ThreadOwnedLookup_0127a580 and FUN_013b1630(GroupID); when a match is found reads a string… |
