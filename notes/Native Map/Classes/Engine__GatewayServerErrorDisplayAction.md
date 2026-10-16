# Class: Engine::GatewayServerErrorDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0124ebd0` | `Engine::GatewayServerErrorDisplayAction::ctor` | 🟢 | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::GatewayServerErrorDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `0124ec60` | `Engine::GatewayServerErrorDisplayAction::DebugName` | ✅ | virtual | Slot [4] of Engine::GatewayServerErrorDisplayAction::vftable. Returns the constant C string 'kGatewayServerErrorDisplayAction : 0 : ..\common\coreclient\engine\network\networkhandlers.cc:255'. |
| `0124ec70` | `Engine::GatewayServerErrorDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `0124fea0` | `— ⚠reject` | ⚪ | virtual | Slot [2] of Engine::GatewayServerErrorDisplayAction::vftable, 396 bytes. If FUN_00f9c010 returns a UI object it reads ValueData entry 0 as an integer (type tag 2, else assert 'mTypeID == kIntegerTypeID' at ValueData.h:0… |
