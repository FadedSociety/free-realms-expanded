# Class: Engine::LobbyInitializeLobbyPlayers

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127e700` | `Engine::LobbyInitializeLobbyPlayers::LobbyInitializeLobbyPlayers` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyInitializeLobbyPlayers::vftable into *param_1, returns this; when the second argument is nonzero calls FUN_012ea4f0. |
| `0127e770` | `Engine::LobbyInitializeLobbyPlayers::GetName` | ✅ | virtual | Returns the literal string InitializeLobbyPlayers (mov eax, imm32; ret). |
| `0127e790` | `Engine::LobbyInitializeLobbyPlayers::GetLogTag` | ✅ | virtual | Returns a pointer to the literal InitializeLobbyPlayers : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:717 (mov eax, imm32; ret). |
| `0127e7a0` | `Engine::LobbyInitializeLobbyPlayers::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `01281230` | `— ⚠reject` | ⚪ | virtual | Virtual slot [2]. When FUN_00f9c010() is nonzero: reads GroupID and GroupCount (FUN_012eb290), reads a text value at DAT_0188ce5c (FUN_012eb250, default DAT_0175b400), calls FUN_012ebc00, sets ListName to Players, Group… |
