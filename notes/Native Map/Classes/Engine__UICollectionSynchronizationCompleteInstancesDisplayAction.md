# Class: Engine::UICollectionSynchronizationCompleteInstancesDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01276470` | `Engine::UICollectionSynchronizationCompleteInstancesDisplayAction::ctor` | ✅ | ctor | Returns this. Stores Engine::IDisplayActionHandler::vftable and then Engine::UICollectionSynchronizationCompleteInstancesDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(thi… |
| `01276500` | `Engine::UICollectionSynchronizationCompleteInstancesDisplayAction::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string kSynchronizationCompleteInstancesDisplayAction : 0 : ..\common\...\collectionhandlers.cc:56. |
| `01276510` | `Engine::UICollectionSynchronizationCompleteInstancesDisplayAction::ScalarDeleti…` | ✅ | dtor | Slot [0] of its vftable. Stores Engine::IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `01278510` | `Engine::UICollectionSynchronizationCompleteInstancesDisplayAction::Handle` | ✅ | virtual | Virtual slot [2] of its vtable (359 bytes). When FUN_00f9c010 returns non-null posts a CloseProgressDialog command. Then allocates a 0x40-byte object with FUN_00d833a6, builds a RequestCollection command through FUN_012… |
