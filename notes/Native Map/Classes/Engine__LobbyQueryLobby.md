# Class: Engine::LobbyQueryLobby

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127dc80` | `Engine::LobbyQueryLobby::LobbyQueryLobby` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyQueryLobby::vftable into *param_1, returns this; when the second argument is nonzero calls FUN_012ea4f0. |
| `0127dcf0` | `Engine::LobbyQueryLobby::GetName` | ✅ | virtual | Returns the literal string QueryLobby (mov eax, imm32; ret). |
| `0127dd10` | `Engine::LobbyQueryLobby::GetLogTag` | ✅ | virtual | Returns a pointer to the literal QueryLobby : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:536 (mov eax, imm32; ret). |
| `0127dd20` | `Engine::LobbyQueryLobby::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `01280820` | `Engine::LobbyQueryLobby::Vslot2` | 🟢 | virtual | Virtual slot [2]. When FUN_0127a580() is nonnull and the GroupID value passes FUN_013b15f0: builds the tag InitializeLobby with FUN_012ec220, sets GroupID, passes FUN_013b15e0 and FUN_013b0b10 results through FUN_012ebe… |
