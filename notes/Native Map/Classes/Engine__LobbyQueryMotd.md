# Class: Engine::LobbyQueryMotd

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127e070` | `Engine::LobbyQueryMotd::LobbyQueryMotd` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyQueryMotd::vftable into *param_1, returns this; when the second argument is nonzero calls FUN_012ea4f0. |
| `0127e0e0` | `Engine::LobbyQueryMotd::GetName` | ✅ | virtual | Returns the literal string QueryMotd (mov eax, imm32; ret). |
| `0127e100` | `Engine::LobbyQueryMotd::GetLogTag` | ✅ | virtual | Returns a pointer to the literal QueryMotd : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:603 (mov eax, imm32; ret). |
| `0127e110` | `Engine::LobbyQueryMotd::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `01280c20` | `Engine::LobbyQueryMotd::Vslot2` | 🟢 | virtual | Virtual slot [2]. When FUN_0127a580() is nonnull: builds the tag QueryMotd with FUN_012ec220, sets GroupID, then writes FUN_013b1800 of GroupID through FUN_012ebe00 into DAT_01765848, then calls FUN_012e5730 and FUN_012… |
