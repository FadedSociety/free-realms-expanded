# Class: Engine::LobbyRequestPlayers

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127d5f0` | `Engine::LobbyRequestPlayers::LobbyRequestPlayers` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyRequestPlayers::vftable into *param_1, returns this; when the second argument is nonzero calls FUN_012ea4f0. |
| `0127d660` | `Engine::LobbyRequestPlayers::GetName` | ✅ | virtual | Returns the literal string RequestPlayers (mov eax, imm32; ret). |
| `0127d680` | `Engine::LobbyRequestPlayers::GetLogTag` | ✅ | virtual | Returns a pointer to the literal RequestPlayers : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:459 (mov eax, imm32; ret). |
| `0127d690` | `Engine::LobbyRequestPlayers::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, returns this, and calls _free(this) when bit 0 of the argument is set. |
| `01280490` | `Engine::LobbyRequestPlayers::Vslot2` | 🟢 | virtual | Virtual slot [2]. When FUN_0127a580() is nonnull: gets a value from FUN_013b1560, looks up the GroupID key with FUN_012eb290, and when FUN_013b15f0 of that is nonzero calls FUN_0127ede0 (InitializeLobbyPlayers) with it,… |
