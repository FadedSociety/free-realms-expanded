# Class: Engine::LobbyJoinLobby

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127d890` | `Engine::LobbyJoinLobby::LobbyJoinLobby` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyJoinLobby::vftable into *param_1, returns this; when the second argument is nonzero calls FUN_012ea4f0. |
| `0127d900` | `Engine::LobbyJoinLobby::GetName` | ✅ | virtual | Returns the literal string JoinLobby (mov eax, imm32; ret). |
| `0127d920` | `Engine::LobbyJoinLobby::GetLogTag` | ✅ | virtual | Returns a pointer to the literal JoinLobby : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:489 (mov eax, imm32; ret). |
| `0127d930` | `Engine::LobbyJoinLobby::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `01280570` | `— ⚠reject` | ⚪ | virtual | Virtual slot [2]. When FUN_0127a580() is nonnull: gets the GroupID key (FUN_012eb290), the key at DAT_018c9f18 and the Password text (FUN_012eb250, default DAT_0175b400), then calls the manager's virtual slot 3 (offset … |
