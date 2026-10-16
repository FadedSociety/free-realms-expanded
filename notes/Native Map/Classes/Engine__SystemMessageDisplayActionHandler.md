# Class: Engine::SystemMessageDisplayActionHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0124f0e0` | `Engine::SystemMessageDisplayActionHandler::ctor` | 🟢 | ctor | Returns this. Stores IDisplayActionHandler::vftable then Engine::SystemMessageDisplayActionHandler::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `0124f170` | `Engine::SystemMessageDisplayActionHandler::DebugName` | ✅ | virtual | Slot [4] of Engine::SystemMessageDisplayActionHandler::vftable. Returns the constant C string 'kSystemMessageDisplayAction : 0 : ..\common\coreclient\engine\network\networkhandlers.cc:41'. |
| `0124f180` | `Engine::SystemMessageDisplayActionHandler::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `0124f3c0` | `Engine::SystemMessageDisplayActionHandler::SetTcgDialogTitle` | ✅ | virtual | Slot [2] of Engine::SystemMessageDisplayActionHandler::vftable. Gets the object from FUN_00f9c010; if non-null it gets a ValueData from FUN_012f8f60(0), requires its type tag (+4) to be 3 (string, otherwise FUN_012f5a60… |
