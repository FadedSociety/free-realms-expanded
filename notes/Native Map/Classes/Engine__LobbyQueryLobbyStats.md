# Class: Engine::LobbyQueryLobbyStats

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127d350` | `Engine::LobbyQueryLobbyStats::LobbyQueryLobbyStats` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyQueryLobbyStats::vftable into *param_1, returns this; when the second argument is nonzero calls FUN_012ea4f0. |
| `0127d3c0` | `Engine::LobbyQueryLobbyStats::GetName` | ✅ | virtual | Returns the literal string QueryLobbyStats (mov eax, imm32; ret). No arguments read. |
| `0127d3e0` | `Engine::LobbyQueryLobbyStats::GetLogTag` | ✅ | virtual | Returns a pointer to the literal QueryLobbyStats : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:417 (mov eax, imm32; ret). |
| `0127d3f0` | `Engine::LobbyQueryLobbyStats::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, returns this, and calls _free(this) when bit 0 of the argument is set. |
| `012803a0` | `Engine::LobbyQueryLobbyStats::Vslot2` | 🟢 | virtual | Virtual slot [2]. Reads GroupID with FUN_012eb290("GroupID",1), takes a count from FUN_012d2200, builds the tag QueryLobbyStats with FUN_012ec220, sets GroupID and UserCount keys, then calls FUN_012e5730 and FUN_012e582… |
