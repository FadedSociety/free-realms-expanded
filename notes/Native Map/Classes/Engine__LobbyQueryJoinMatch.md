# Class: Engine::LobbyQueryJoinMatch

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127e310` | `Engine::LobbyQueryJoinMatch::LobbyQueryJoinMatch` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyQueryJoinMatch::vftable into *param_1, returns this; when the second argument is nonzero calls FUN_012ea4f0. |
| `0127e380` | `Engine::LobbyQueryJoinMatch::GetName` | ✅ | virtual | Returns the literal string QueryJoinMatch (mov eax, imm32; ret). |
| `0127e3a0` | `Engine::LobbyQueryJoinMatch::GetLogTag` | ✅ | virtual | Returns a pointer to the literal QueryJoinMatch : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:631 (mov eax, imm32; ret). |
| `0127e3b0` | `Engine::LobbyQueryJoinMatch::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `01280d60` | `Engine::LobbyQueryJoinMatch::Vslot2` | 🟢 | virtual | Virtual slot [2]. When FUN_0127a580() is nonnull and FUN_013b1630 of GroupID is nonzero: calls FUN_0126dad0() and, when it is nonnull, copies the string returned by its virtual slot 23 (offset 0x5c) as the selected deck… |
