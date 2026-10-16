# Class: Engine::CollectionSynchronizationStartInstancesDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01277f30` | `Engine::CollectionSynchronizationStartInstancesDisplayAction::ctor` | ✅ | ctor | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::CollectionSynchronizationStartInstancesDisplayAction::vftable into *this; when the int argument is nonzero calls FUN_012ea300(this). |
| `01277fc0` | `Engine::CollectionSynchronizationStartInstancesDisplayAction::DebugName` | ✅ | virtual | Returns the constant C string kSynchronizationStartInstancesDisplayAction : 0 : ... (6 bytes: mov eax, imm32; ret). Virtual slot [4] of its vtable. |
| `01277fd0` | `Engine::CollectionSynchronizationStartInstancesDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `012782c0` | `— ⚠reject` | ⚪ | virtual | Virtual slot [2] of its vtable (297 bytes). Reads ValueData argument 2 as an integer (type tag 2, else assert mTypeID == kIntegerTypeID at ValueData.h:0x129 and 0). When FUN_00f9c010 returns non-null, builds the Tcg_syn… |
