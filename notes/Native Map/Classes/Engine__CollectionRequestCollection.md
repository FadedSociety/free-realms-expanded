# Class: Engine::CollectionRequestCollection

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01276df0` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::CollectionRequestCollection::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01276e60` | `Engine::CollectionRequestCollection::ActionName` | ✅ | virtual | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string RequestCollection. |
| `01276e80` | `Engine::CollectionRequestCollection::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string RequestCollection : 0 : ..\common\...\collectionhandlers.cc:176. |
| `01276e90` | `Engine::CollectionRequestCollection::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `01278b50` | `Engine::CollectionRequestCollection::Handle` | ✅ | virtual | Virtual slot [2] of its vtable (308 bytes). Reads the bool ShowUnowned through FUN_012eb310; calls FUN_0126db60 and its vtable slot +0x24; when that returns true builds the InitializeCollection command with FUN_012ec220… |
