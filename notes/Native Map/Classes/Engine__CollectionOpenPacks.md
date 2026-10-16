# Class: Engine::CollectionOpenPacks

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01278030` | `Engine::CollectionOpenPacks::ctor` | ✅ | ctor | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::CollectionOpenPacks::vftable into *this; when the int argument is nonzero calls FUN_012ea4f0(this). |
| `012780a0` | `Engine::CollectionOpenPacks::ActionName` | ✅ | virtual | Returns the constant C string OpenPacks (6 bytes: mov eax, imm32; ret). Virtual slot [1] of Engine::CollectionOpenPacks::vftable. |
| `012780c0` | `Engine::CollectionOpenPacks::DebugName` | ✅ | virtual | Returns the constant C string OpenPacks : 0 : ...collectionhandlers.cc:127 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of Engine::CollectionOpenPacks::vftable. |
| `012780d0` | `Engine::CollectionOpenPacks::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `01278170` | `Engine::CollectionOpenPacks::Handle` | ✅ | virtual | Virtual slot [2] of Engine::CollectionOpenPacks::vftable (79 bytes). Calls FUN_012e0ba0, then FUN_0126db60; when non-null builds ArchetypeID and Quantity keys via FUN_012eb290 and calls its vtable slot +0xc with (Archet… |
