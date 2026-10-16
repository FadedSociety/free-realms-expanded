# Class: Engine::CollectionResetFilter

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01277b50` | `Engine::CollectionResetFilter::ctor` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::CollectionResetFilter::vftable into *this; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01277bc0` | `Engine::CollectionResetFilter::ActionName` | ✅ | virtual | Returns the constant C string ResetFilter (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::CollectionResetFilter::vftable. |
| `01277be0` | `Engine::CollectionResetFilter::DebugName` | ✅ | virtual | Returns the constant C string ResetFilter : 0 : ...collectionhandlers.cc:295 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::CollectionResetFilter::vftable. |
| `01277bf0` | `Engine::CollectionResetFilter::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
