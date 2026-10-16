# Class: Engine::CollectionAddFilter

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01277730` | `Engine::CollectionAddFilter::ctor` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::CollectionAddFilter::vftable into *this; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `012777a0` | `Engine::CollectionAddFilter::ActionName` | ✅ | virtual | Returns the constant C string AddFilter (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::CollectionAddFilter::vftable. |
| `012777c0` | `Engine::CollectionAddFilter::DebugName` | ✅ | virtual | Returns the constant C string AddFilter : 0 : ...collectionhandlers.cc:277 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::CollectionAddFilter::vftable. |
| `012777d0` | `Engine::CollectionAddFilter::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
