# Class: Engine::CollectionQueryMoz

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012775e0` | `Engine::CollectionQueryMoz::ctor` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::CollectionQueryMoz::vftable into *this; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01277650` | `Engine::CollectionQueryMoz::ActionName` | ✅ | virtual | Returns the constant C string QueryMoz (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::CollectionQueryMoz::vftable. |
| `01277670` | `Engine::CollectionQueryMoz::DebugName` | ✅ | virtual | Returns the constant C string QueryMoz : 0 : ...collectionhandlers.cc:261 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::CollectionQueryMoz::vftable. |
| `01277680` | `Engine::CollectionQueryMoz::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `01278ff0` | `— ⚠reject` | ⚪ | virtual | Virtual slot [2] of its vtable (300 bytes). Reads the ArchetypeID value through FUN_012eb290, builds the QueryMoz command with FUN_012ec220, sets ArchetypeID through FUN_012ebe40 and FUN_013ab330, then posts through FUN… |
