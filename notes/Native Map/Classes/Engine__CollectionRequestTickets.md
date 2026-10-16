# Class: Engine::CollectionRequestTickets

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012771e0` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::CollectionRequestTickets::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01277250` | `Engine::CollectionRequestTickets::ActionName` | ✅ | virtual | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string RequestTickets. |
| `01277270` | `Engine::CollectionRequestTickets::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string RequestTickets : 0 : ..\common\...\collectionhandlers.cc:226. |
| `01277280` | `Engine::CollectionRequestTickets::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
