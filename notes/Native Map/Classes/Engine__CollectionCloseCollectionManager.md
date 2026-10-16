# Class: Engine::CollectionCloseCollectionManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01277e10` | `Engine::CollectionCloseCollectionManager::ctor` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::CollectionCloseCollectionManager::vftable into *this; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01277e80` | `Engine::CollectionCloseCollectionManager::ActionName` | ✅ | virtual | Returns the constant C string CloseCollectionManager (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::CollectionCloseCollectionManager::vftable. |
| `01277ea0` | `Engine::CollectionCloseCollectionManager::DebugName` | ✅ | virtual | Returns the constant C string CloseCollectionManager : 0 : ...collectionhandlers.cc:307 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::CollectionCloseCollectionManager::vftable. |
| `01277eb0` | `Engine::CollectionCloseCollectionManager::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
