# Class: Engine::LobbyChangeScreenByGroupID

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127e5b0` | `Engine::LobbyChangeScreenByGroupID::LobbyChangeScreenByGroupID` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyChangeScreenByGroupID::vftable into *param_1, returns this; when the second argument is nonzero calls FUN_012ea4f0. |
| `0127e620` | `Engine::LobbyChangeScreenByGroupID::GetName` | ✅ | virtual | Returns the literal string ChangeScreenByGroupID (mov eax, imm32; ret). |
| `0127e640` | `Engine::LobbyChangeScreenByGroupID::GetLogTag` | ✅ | virtual | Returns a pointer to the literal ChangeScreenByGroupID : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:685 (mov eax, imm32; ret). |
| `0127e650` | `Engine::LobbyChangeScreenByGroupID::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `01281090` | `Engine::LobbyChangeScreenByGroupID::Vslot2` | 🟢 | virtual | Virtual slot [2]. When FUN_0127a580() is nonnull: reads GroupID through FUN_013b1560 and FUN_012eb290. Chooses the screen name by the group value: 1 gives UI::MenuScreen, 2 gives UI::LobbyScreen, 5 gives UI::TournamentS… |
