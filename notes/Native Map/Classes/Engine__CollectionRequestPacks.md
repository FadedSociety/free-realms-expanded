# Class: Engine::CollectionRequestPacks

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01276f40` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::CollectionRequestPacks::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01276fb0` | `Engine::CollectionRequestPacks::ActionName` | ✅ | virtual | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string RequestPacks. |
| `01276fd0` | `Engine::CollectionRequestPacks::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string RequestPacks : 0 : ..\common\...\collectionhandlers.cc:191. |
| `01276fe0` | `Engine::CollectionRequestPacks::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `01278c90` | `— ⚠reject` | ⚪ | virtual | Virtual slot [2] of its vtable (267 bytes). Calls FUN_012e0ba0 and FUN_0126db60; when non-null builds the RespondPacks command with FUN_012ec220, attaches a value built by FUN_013ae510, and posts it through FUN_012e5730… |
