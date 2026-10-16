# Class: Engine::CollectionRequestCardData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01277490` | `Engine::CollectionRequestCardData::ctor` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::CollectionRequestCardData::vftable into *this; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01277500` | `Engine::CollectionRequestCardData::ActionName` | ✅ | virtual | Returns the constant C string RequestCardData (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::CollectionRequestCardData::vftable. |
| `01277520` | `Engine::CollectionRequestCardData::DebugName` | ✅ | virtual | Returns the constant C string RequestCardData : 0 : ...collectionhandlers.cc:245 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::CollectionRequestCardData::vftable. |
| `01277530` | `Engine::CollectionRequestCardData::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `01278eb0` | `— ⚠reject` | ⚪ | virtual | Virtual slot [2] of its vtable (300 bytes). Reads the ArchetypeID value through FUN_012eb290, calls FUN_0126db60, builds the SendCardData command with FUN_012ec220, sets ArchetypeID through FUN_012ebe40 and FUN_013ab330… |
