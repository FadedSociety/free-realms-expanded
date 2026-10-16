# Class: Engine::EngineUpdateDataVersionDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0124e3a0` | `Engine::EngineUpdateDataVersionDisplayAction::ctor` | 🟢 | ctor | Returns this. Stores the base Engine::IDisplayActionHandler::vftable and then Engine::EngineUpdateDataVersionDisplayAction::vftable into the object; when the int argument is nonzero it also calls FUN_012ea300(this). |
| `0124e430` | `Engine::EngineUpdateDataVersionDisplayAction::DebugName` | ✅ | virtual | Virtual method in slot [4] of Engine::EngineUpdateDataVersionDisplayAction::vftable. Returns the constant C string 'kUpdateDataVersionDisplayAction : 0 : ..\common\coreclient\engine\network\networkhandlers.cc:52'. |
| `0124e440` | `Engine::EngineUpdateDataVersionDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of Engine::EngineUpdateDataVersionDisplayAction::vftable. Stores Engine::IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; retu… |
| `01250230` | `Engine::EngineUpdateDataVersionDisplayAction::CacheDataVersion` | ✅ | virtual | Slot [2] of Engine::EngineUpdateDataVersionDisplayAction::vftable, 1362 bytes. Gets the thread-owned object (FUN_0124f2e0); reads ValueData entry 0 as the integer data version (type tag 2, else assert kIntegerTypeID at … |
