# Class: Engine::GatewayServerSuccessDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0124ea80` | `Engine::GatewayServerSuccessDisplayAction::ctor` | 🟢 | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::GatewayServerSuccessDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `0124eb10` | `Engine::GatewayServerSuccessDisplayAction::DebugName` | ✅ | virtual | Slot [4] of Engine::GatewayServerSuccessDisplayAction::vftable. Returns the constant C string 'kGatewayServerSuccessDisplayAction : 0 : ..\common\coreclient\engine\network\networkhandlers.cc:241'. |
| `0124eb20` | `Engine::GatewayServerSuccessDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `0124fda0` | `Engine::GatewayServerSuccessDisplayAction::ApplyConnectionSucceeded` | ✅ | virtual | Slot [2] of Engine::GatewayServerSuccessDisplayAction::vftable, 242 bytes. If FUN_00f9c010 returns a UI object it sets the value 'ConnectionSucceeded' to 1 through FUN_012ebe80, sets 'Explanation' to 'Tcg_login.loggingI… |
