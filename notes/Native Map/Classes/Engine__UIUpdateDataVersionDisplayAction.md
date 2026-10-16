# Class: Engine::UIUpdateDataVersionDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0124e640` | `Engine::UIUpdateDataVersionDisplayAction::ctor` | 🟢 | ctor | Returns this. Stores the base Engine::IDisplayActionHandler::vftable and then Engine::UIUpdateDataVersionDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `0124e6d0` | `Engine::UIUpdateDataVersionDisplayAction::DebugName` | ✅ | virtual | Slot [4] of Engine::UIUpdateDataVersionDisplayAction::vftable. Returns the constant C string 'kUpdateDataVersionDisplayAction : 0 : ..\common\coreclient\engine\network\networkhandlers.cc:119'. |
| `0124e6e0` | `Engine::UIUpdateDataVersionDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of Engine::UIUpdateDataVersionDisplayAction::vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `0124f3a0` | `Engine::UIUpdateDataVersionDisplayAction::ApplyGuarded ? ⚠downgrade` | 🟡 | virtual | 20-byte body: calls FUN_012e0ba0, then FUN_00f9c010 (result discarded), then returns 1 with ret 8 (callee pops 8 bytes of arguments). |
