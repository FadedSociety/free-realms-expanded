# Class: Engine::LobbyQueryLobbyStatus

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127e460` | `Engine::LobbyQueryLobbyStatus::LobbyQueryLobbyStatus` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyQueryLobbyStatus::vftable into *param_1, returns this; when the second argument is nonzero calls FUN_012ea4f0. |
| `0127e4d0` | `Engine::LobbyQueryLobbyStatus::GetName` | ✅ | virtual | Returns the literal string QueryLobbyStatus (mov eax, imm32; ret). |
| `0127e4f0` | `Engine::LobbyQueryLobbyStatus::GetLogTag` | ✅ | virtual | Returns a pointer to the literal QueryLobbyStatus : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:662 (mov eax, imm32; ret). |
| `0127e500` | `Engine::LobbyQueryLobbyStatus::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `01280f60` | `Engine::LobbyQueryLobbyStatus::Vslot2` | 🟢 | virtual | Virtual slot [2]. When FUN_0127a580() is nonnull: reads GroupID (FUN_012eb290); if FUN_013b15f0 of it is nonzero, sets a flag from FUN_0130ae30(FUN_012d08f0()). Builds the tag UpdateLobbyStatus with FUN_012ec220, sets L… |
