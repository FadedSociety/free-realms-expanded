# Class: Engine::NetworkConnectDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0124ee70` | `Engine::NetworkConnectDisplayAction::ctor` | 🟢 | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::NetworkConnectDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `0124ef00` | `Engine::NetworkConnectDisplayAction::DebugName` | ✅ | virtual | Slot [4] of Engine::NetworkConnectDisplayAction::vftable. Returns the constant C string 'kNetworkConnectDisplayAction : 0 : ..\common\coreclient\engine\network\networkhandlers.cc:287'. |
| `0124ef10` | `Engine::NetworkConnectDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
