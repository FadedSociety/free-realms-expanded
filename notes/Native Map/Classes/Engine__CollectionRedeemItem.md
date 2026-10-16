# Class: Engine::CollectionRedeemItem

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01276b50` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::CollectionRedeemItem::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01276bc0` | `Engine::CollectionRedeemItem::ActionName` | ✅ | virtual | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string RedeemItem. |
| `01276be0` | `Engine::CollectionRedeemItem::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string RedeemItem : 0 : ..\common\...\collectionhandlers.cc:142. |
| `01276bf0` | `Engine::CollectionRedeemItem::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `012781c0` | `Engine::CollectionRedeemItem::Handle` | ✅ | virtual | Virtual slot [2] of Engine::CollectionRedeemItem::vftable (129 bytes). Calls FUN_012e0ba0, then FUN_0126db60; when non-null builds ArchetypeID and ChosenArchetypeID keys and the IsConversion and IsTradable flags, then c… |
