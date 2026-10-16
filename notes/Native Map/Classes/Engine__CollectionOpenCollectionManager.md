# Class: Engine::CollectionOpenCollectionManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01277cb0` | `Engine::CollectionOpenCollectionManager::ctor` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::CollectionOpenCollectionManager::vftable into *this; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `01277d20` | `Engine::CollectionOpenCollectionManager::ActionName` | ✅ | virtual | Returns the constant C string OpenCollectionManager (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::CollectionOpenCollectionManager::vftable. |
| `01277d40` | `Engine::CollectionOpenCollectionManager::DebugName` | ✅ | virtual | Returns the constant C string OpenCollectionManager : 0 : ...collectionhandlers.cc:301 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::CollectionOpenCollectionManager::vftable. |
| `01277d50` | `Engine::CollectionOpenCollectionManager::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
