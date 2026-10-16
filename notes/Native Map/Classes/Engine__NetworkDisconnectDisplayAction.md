# Class: Engine::NetworkDisconnectDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0124efd0` | `Engine::NetworkDisconnectDisplayAction::ctor` | 🟢 | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::NetworkDisconnectDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `0124f060` | `Engine::NetworkDisconnectDisplayAction::DebugName` | ✅ | virtual | Slot [4] of Engine::NetworkDisconnectDisplayAction::vftable. Returns the constant C string 'kNetworkDisconnectDisplayAction : 0 : ..\common\coreclient\engine\network\networkhandlers.cc:293'. |
| `0124f070` | `Engine::NetworkDisconnectDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `01250140` | `— ⚠reject` | ⚪ | virtual | Slot [2] of Engine::NetworkDisconnectDisplayAction::vftable, 227 bytes. Allocates 0x40 bytes through FUN_00d833a6; if the allocation succeeds it builds the string 'CloseClient' and creates an action through FUN_012ec110… |
