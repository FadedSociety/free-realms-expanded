# Class: Engine::CollectionProcessFulfillment

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01276ca0` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::CollectionProcessFulfillment::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01276d10` | `Engine::CollectionProcessFulfillment::ActionName` | ✅ | virtual | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string ProcessFulfillment. |
| `01276d30` | `Engine::CollectionProcessFulfillment::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string ProcessFulfillment : 0 : ..\common\...\collectionhandlers.cc:160. |
| `01276d40` | `Engine::CollectionProcessFulfillment::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `01278250` | `Engine::CollectionProcessFulfillment::Handle` | ✅ | virtual | Virtual slot [2] of Engine::CollectionProcessFulfillment::vftable (104 bytes). Calls FUN_012e0ba0, then FUN_0126db60; when non-null builds ArchetypeID and Quantity keys and the IsTradable flag, then calls its vtable slo… |
