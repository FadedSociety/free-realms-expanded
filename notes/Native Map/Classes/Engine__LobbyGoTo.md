# Class: Engine::LobbyGoTo

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127ea80` | `Engine::LobbyGoTo::LobbyGoTo` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyGoTo::vftable into *param_1, returns this; when the second argument is nonzero calls FUN_012ea4f0. |
| `0127eaf0` | `Engine::LobbyGoTo::GetName` | ✅ | virtual | Returns the address of static data DAT_018caa50 (mov eax, imm32; ret). The bytes at that address begin with the NUL-terminated text GoTo, so this returns a const char* name. |
| `0127eb10` | `Engine::LobbyGoTo::GetLogTag` | ✅ | virtual | Returns a pointer to the literal GoTo : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:392 (mov eax, imm32; ret). |
| `0127eb20` | `Engine::LobbyGoTo::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `012801b0` | `— ⚠reject` | ⚪ | virtual | Virtual slot [2]. When FUN_012eb350("Lobby") is true and FUN_0127a580() is nonnull: gets the Lobby key (FUN_012eb290), the key at DAT_018c9f18 and the Password text (FUN_012eb250, default DAT_0175b400), calls the manage… |
