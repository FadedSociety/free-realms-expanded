# Class: Engine::LobbySetMatchStateDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127e980` | `Engine::LobbySetMatchStateDisplayAction::LobbySetMatchStateDisplayAction` | ✅ | ctor | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::LobbySetMatchStateDisplayAction::vftable into *param_1, returns this; when the second argument is nonzero calls FUN_012ea300. |
| `0127ea10` | `Engine::LobbySetMatchStateDisplayAction::GetLogTag` | ✅ | virtual | Returns a pointer to the literal kSetMatchStateDisplayAction : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:123 (mov eax, imm32; ret). |
| `0127ea20` | `Engine::LobbySetMatchStateDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: EH frame, stores Engine::IDisplayActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `0127f1d0` | `Engine::LobbySetMatchStateDisplayAction::Vslot2` | 🟢 | virtual | Virtual slot [2]. Returns 1 when FUN_00f9c010() is zero. Otherwise reads ValueData entries 0 and 1 (each must be integer type 2, else the assert text mTypeID == kIntegerTypeID at ValueData.h line 0x129 is raised and 0 i… |
