# Class: Engine::CollectionSynchronizationStartDeliveryDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01276710` | `Engine::CollectionSynchronizationStartDeliveryDisplayAction::ctor` | ✅ | ctor | Returns this. Stores Engine::IDisplayActionHandler::vftable and then Engine::CollectionSynchronizationStartDeliveryDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `012767a0` | `Engine::CollectionSynchronizationStartDeliveryDisplayAction::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string kSynchronizationStartDeliveryDisplayAction : 0 : ..\common\...\collectionhandlers.cc:85. |
| `012767b0` | `Engine::CollectionSynchronizationStartDeliveryDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Engine::IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `01278680` | `Engine::CollectionSynchronizationStartDeliveryDisplayAction::Handle` | ✅ | virtual | Virtual slot [2] of its vtable (802 bytes). Reads ValueData entries 0 to 6 with type checks (integers 0, 1, 3, 5, 6; boolean 2; string 4). Calls FUN_0126db60 and its vtable slot +0x14 with the seven values. When the boo… |
