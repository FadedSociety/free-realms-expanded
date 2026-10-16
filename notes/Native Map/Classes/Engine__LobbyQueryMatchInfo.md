# Class: Engine::LobbyQueryMatchInfo

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127ddd0` | `Engine::LobbyQueryMatchInfo::LobbyQueryMatchInfo` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyQueryMatchInfo::vftable into *param_1, returns this; when the second argument is nonzero calls FUN_012ea4f0. |
| `0127de40` | `Engine::LobbyQueryMatchInfo::GetName` | ✅ | virtual | Returns the literal string QueryMatchInfo (mov eax, imm32; ret). |
| `0127de60` | `Engine::LobbyQueryMatchInfo::GetLogTag` | ✅ | virtual | Returns a pointer to the literal QueryMatchInfo : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:558 (mov eax, imm32; ret). |
| `0127de70` | `Engine::LobbyQueryMatchInfo::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `01280980` | `Engine::LobbyQueryMatchInfo::Vslot2` | 🟢 | virtual | Virtual slot [2]. When FUN_0127a580() is nonnull and FUN_013b1630 of the GroupID value is nonzero: builds the tag OpenDialog with FUN_012ec220, sets GroupID, passes a value from FUN_013af190 through FUN_012ebe00 on DAT_… |
