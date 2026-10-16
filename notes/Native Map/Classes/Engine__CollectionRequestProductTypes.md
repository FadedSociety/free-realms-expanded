# Class: Engine::CollectionRequestProductTypes

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01277090` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::CollectionRequestProductTypes::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01277100` | `Engine::CollectionRequestProductTypes::ActionName` | ✅ | virtual | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string RequestProductTypes. |
| `01277120` | `Engine::CollectionRequestProductTypes::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string RequestProductTypes : 0 : ..\common\...\collectionhandlers.cc:204. |
| `01277130` | `Engine::CollectionRequestProductTypes::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `01279130` | `— ⚠reject` | ⚪ | virtual | Virtual slot [2] of its vtable (496 bytes). Reads a string at key DAT_0175b400 through FUN_012eb070, reads an integer count through FUN_012eb0a0 and fills a vector with the integers that follow; then calls FUN_0126db60 … |
