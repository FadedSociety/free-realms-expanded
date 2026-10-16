# Class: Engine::FRTWUILobbyRequestQuickJoin

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01279aa0` | `Engine::FRTWUILobbyRequestQuickJoin::ctor` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWUILobbyRequestQuickJoin::vftable into *this; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01279b10` | `Engine::FRTWUILobbyRequestQuickJoin::ActionName` | ✅ | virtual | Returns the constant C string RequestQuickJoin (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::FRTWUILobbyRequestQuickJoin::vftable. |
| `01279b30` | `Engine::FRTWUILobbyRequestQuickJoin::DebugName` | ✅ | virtual | Returns the constant C string RequestQuickJoin : 0 : .\engine\lobby\frtwlobbyhandlers.cc:206 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::FRTWUILobbyRequestQuickJoin::vftable. |
| `01279b40` | `Engine::FRTWUILobbyRequestQuickJoin::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `0127b170` | `— ⚠reject` | ⚪ | virtual | Virtual slot [2] of Engine::FRTWUILobbyRequestQuickJoin::vftable (2506 bytes). Builds the string /debug/debug_mode and calls the unlabelled helpers FUN_012e4410 and FUN_012e3460, then uses the lobby object from ThreadOw… |
