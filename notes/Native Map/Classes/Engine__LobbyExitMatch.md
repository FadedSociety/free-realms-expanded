# Class: Engine::LobbyExitMatch

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127e1c0` | `Engine::LobbyExitMatch::LobbyExitMatch` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyExitMatch::vftable into *param_1, returns this; when the second argument is nonzero calls FUN_012ea4f0. |
| `0127e230` | `Engine::LobbyExitMatch::GetName` | ✅ | virtual | Returns the literal string ExitMatch (mov eax, imm32; ret). |
| `0127e250` | `Engine::LobbyExitMatch::GetLogTag` | ✅ | virtual | Returns a pointer to the literal ExitMatch : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:620 (mov eax, imm32; ret). |
| `0127e260` | `Engine::LobbyExitMatch::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `0127ec80` | `Engine::LobbyExitMatch::Vslot2` | 🟢 | virtual | Virtual slot [2]: calls FUN_012e0ba0(), gets a manager pointer from FUN_0127a580(); if non-null, calls the manager's virtual slot at offset 0x1c. Returns 1. |
