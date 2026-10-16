# Class: Engine::LobbyRequestSubLobbyPlayers

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127d4a0` | `Engine::LobbyRequestSubLobbyPlayers::LobbyRequestSubLobbyPlayers` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyRequestSubLobbyPlayers::vftable into *param_1, returns this; when the second argument is nonzero calls FUN_012ea4f0. |
| `0127d510` | `Engine::LobbyRequestSubLobbyPlayers::GetName` | ✅ | virtual | Returns the literal string RequestSubLobbyPlayers (mov eax, imm32; ret). |
| `0127d530` | `Engine::LobbyRequestSubLobbyPlayers::GetLogTag` | ✅ | virtual | Returns a pointer to the literal RequestSubLobbyPlayers : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:430 (mov eax, imm32; ret). |
| `0127d540` | `Engine::LobbyRequestSubLobbyPlayers::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: EH frame with state -1, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set, returns this. |
| `01281500` | `Engine::LobbyRequestSubLobbyPlayers::Vslot2` | 🟢 | virtual | Virtual slot [2]. When FUN_0127a580() is nonnull and the GroupID value passes FUN_013b15f0: loops over five fixed codes 3, 1, 6, 10, 11, collecting entries with FUN_0130a060 into a local vector; for each collected entry… |
