# Class: Engine::LobbyRequestMatches

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127d740` | `Engine::LobbyRequestMatches::LobbyRequestMatches` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyRequestMatches::vftable into *param_1, returns this; when the second argument is nonzero calls FUN_012ea4f0. |
| `0127d7b0` | `Engine::LobbyRequestMatches::GetName` | ✅ | virtual | Returns the literal string RequestMatches (mov eax, imm32; ret). |
| `0127d7d0` | `Engine::LobbyRequestMatches::GetLogTag` | ✅ | virtual | Returns a pointer to the literal RequestMatches : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:474 (mov eax, imm32; ret). |
| `0127d7e0` | `Engine::LobbyRequestMatches::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: EH frame with state -1, stores Scaleform::IScaleformActionHandler::vftable into *param_1, and calls _free(this) when bit 0 of the argument is set. |
| `01280500` | `Engine::LobbyRequestMatches::Vslot2` | 🟢 | virtual | Virtual slot [2]. When FUN_0127a580() is nonnull: gets a value from FUN_013b1560, looks up the GroupID key with FUN_012eb290, and when FUN_013b15f0 of that is nonzero calls FUN_0127ecb0 (InitializeLobbyMatches) with it … |
