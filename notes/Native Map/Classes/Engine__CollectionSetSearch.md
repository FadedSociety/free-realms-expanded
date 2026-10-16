# Class: Engine::CollectionSetSearch

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012779f0` | `Engine::CollectionSetSearch::ctor` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::CollectionSetSearch::vftable into *this; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01277a60` | `Engine::CollectionSetSearch::ActionName` | ✅ | virtual | Returns the constant C string SetSearch (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::CollectionSetSearch::vftable. |
| `01277a80` | `Engine::CollectionSetSearch::DebugName` | ✅ | virtual | Returns the constant C string SetSearch : 0 : ...collectionhandlers.cc:289 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::CollectionSetSearch::vftable. |
| `01277a90` | `Engine::CollectionSetSearch::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
