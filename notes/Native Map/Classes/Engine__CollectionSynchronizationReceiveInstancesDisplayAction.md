# Class: Engine::CollectionSynchronizationReceiveInstancesDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01276320` | `Engine::CollectionSynchronizationReceiveInstancesDisplayAction::ctor` | ✅ | ctor | Returns this. Stores Engine::IDisplayActionHandler::vftable and then Engine::CollectionSynchronizationReceiveInstancesDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `012763b0` | `Engine::CollectionSynchronizationReceiveInstancesDisplayAction::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string kSynchronizationReceiveInstancesDisplayAction : 0 : ..\common\...\collectionhandlers.cc:40. |
| `012763c0` | `Engine::CollectionSynchronizationReceiveInstancesDisplayAction::ScalarDeletingD…` | ✅ | dtor | Slot [0] of its vftable. Stores Engine::IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `012783f0` | `— ⚠reject` | ⚪ | virtual | Virtual slot [2] of its vtable (279 bytes). Reads ValueData argument 1 as an integer (type tag 2, else assert mTypeID == kIntegerTypeID at ValueData.h:0x129 and 0). When FUN_00f9c010 returns non-null, sets the Progress … |
