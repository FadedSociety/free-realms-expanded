# Class: Engine::LoginResultsDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0124e930` | `Engine::LoginResultsDisplayAction::ctor` | 🟢 | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::LoginResultsDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `0124e9c0` | `Engine::LoginResultsDisplayAction::DebugName` | ✅ | virtual | Slot [4] of Engine::LoginResultsDisplayAction::vftable. Returns the constant C string 'kLoginResultsDisplayAction : 0 : ..\common\coreclient\engine\network\networkhandlers.cc:159'. |
| `0124e9d0` | `Engine::LoginResultsDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of Engine::LoginResultsDisplayAction::vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `0124f780` | `— → Engine::LoginResultsDisplayAction::Invoke ⚠reject` | ⚪ | virtual | Slot [2] of Engine::LoginResultsDisplayAction::vftable, 1548 bytes. Reads ValueData entry 0 as a boolean (type tag 1, else assert 'mTypeID == kBooleanTypeID' at ValueData.h:0x12f) for login success and entry 1 as an int… |
