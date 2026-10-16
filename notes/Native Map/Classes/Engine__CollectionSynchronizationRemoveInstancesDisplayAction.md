# Class: Engine::CollectionSynchronizationRemoveInstancesDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01276860` | `Engine::CollectionSynchronizationRemoveInstancesDisplayAction::ctor` | ✅ | ctor | Returns this. Stores Engine::IDisplayActionHandler::vftable and then Engine::CollectionSynchronizationRemoveInstancesDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `012768f0` | `Engine::CollectionSynchronizationRemoveInstancesDisplayAction::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string kSynchronizationRemoveInstancesDisplayAction : 0 : ..\common\...\collectionhandlers.cc:113. |
| `01276900` | `Engine::CollectionSynchronizationRemoveInstancesDisplayAction::ScalarDeletingDt…` | ✅ | dtor | Slot [0] of its vftable. Stores Engine::IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `012789b0` | `— ⚠reject` | ⚪ | virtual | Virtual slot [2] of its vtable (233 bytes). Allocates a 0x40-byte object with FUN_00d833a6, builds a RequestCollection command through FUN_012ec110 and posts it with FUN_012e5730 and FUN_012e5820; returns 1 (0 when the … |
