# Class: Engine::CollectionReloadCollectionDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012769b0` | `Engine::CollectionReloadCollectionDisplayAction::ctor` | ✅ | ctor | Returns this. Stores Engine::IDisplayActionHandler::vftable and then Engine::CollectionReloadCollectionDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `01276a40` | `Engine::CollectionReloadCollectionDisplayAction::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string kReloadCollectionDisplayAction : 0 : ..\common\...\collectionhandlers.cc:119. |
| `01276a50` | `Engine::CollectionReloadCollectionDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Engine::IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `01278aa0` | `Engine::CollectionReloadCollectionDisplayAction::Handle` | ✅ | virtual | Virtual slot [2] of its vtable (169 bytes). Builds the string UpdateCollection and passes it to FUN_012ec220, then posts the result through FUN_012e5730 and FUN_012e5820; returns 1. |
