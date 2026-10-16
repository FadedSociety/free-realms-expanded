# Class: Engine::LobbyOnJoinedGroup

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127e850` | `Engine::LobbyOnJoinedGroup::LobbyOnJoinedGroup` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyOnJoinedGroup::vftable into *param_1, returns this; when the second argument is nonzero calls FUN_012ea4f0. |
| `0127e8c0` | `Engine::LobbyOnJoinedGroup::GetName` | ✅ | virtual | Returns the literal string OnJoinedGroup (mov eax, imm32; ret). |
| `0127e8e0` | `Engine::LobbyOnJoinedGroup::GetLogTag` | ✅ | virtual | Returns a pointer to the literal OnJoinedGroup : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:736 (mov eax, imm32; ret). |
| `0127e8f0` | `Engine::LobbyOnJoinedGroup::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `01281410` | `Engine::LobbyOnJoinedGroup::Vslot2` | 🟢 | virtual | Virtual slot [2]. When FUN_00f9c010() is nonzero: calls FUN_012ebc00, sets GroupID (FUN_012eb290 lookup) and passes the tag OnJoinedGroup to FUN_01230b70. Returns 1. |
