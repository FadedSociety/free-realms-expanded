# Class: Engine::LobbyLeaveLobby

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127db30` | `Engine::LobbyLeaveLobby::LobbyLeaveLobby` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyLeaveLobby::vftable into *param_1, returns this; when the second argument is nonzero calls FUN_012ea4f0. |
| `0127dba0` | `Engine::LobbyLeaveLobby::GetName` | ✅ | virtual | Returns the literal string LeaveLobby (mov eax, imm32; ret). |
| `0127dbc0` | `Engine::LobbyLeaveLobby::GetLogTag` | ✅ | virtual | Returns a pointer to the literal LeaveLobby : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:522 (mov eax, imm32; ret). |
| `0127dbd0` | `Engine::LobbyLeaveLobby::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `0127ec40` | `Engine::LobbyLeaveLobby::Vslot2` | 🟢 | virtual | Virtual slot [2]: calls FUN_012e0ba0(), gets a manager pointer from FUN_0127a580(); if non-null, builds the string GroupID via FUN_012eb290("GroupID",0) and calls the manager's virtual slot at offset 0x10 with it. Retur… |
