# Class: Engine::CollectionRemoveFilter

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01277890` | `Engine::CollectionRemoveFilter::ctor` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::CollectionRemoveFilter::vftable into *this; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01277900` | `Engine::CollectionRemoveFilter::ActionName` | ✅ | virtual | Returns the constant C string RemoveFilter (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::CollectionRemoveFilter::vftable. |
| `01277920` | `Engine::CollectionRemoveFilter::DebugName` | ✅ | virtual | Returns the constant C string RemoveFilter : 0 : ...collectionhandlers.cc:283 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::CollectionRemoveFilter::vftable. |
| `01277930` | `Engine::CollectionRemoveFilter::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
