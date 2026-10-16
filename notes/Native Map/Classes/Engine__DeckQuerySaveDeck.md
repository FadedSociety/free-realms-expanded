# Class: Engine::DeckQuerySaveDeck

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01273360` | `Engine::DeckQuerySaveDeck::ctor` | ✅ | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::DeckQuerySaveDeck::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `012733d0` | `Engine::DeckQuerySaveDeck::ActionName` | ✅ | virtual | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string QuerySaveDeck. |
| `012733f0` | `Engine::DeckQuerySaveDeck::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string QuerySaveDeck : 0 : ..\common\...\deckhandlers.cc:226. |
| `01273400` | `Engine::DeckQuerySaveDeck::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `012751e0` | `Engine::DeckQuerySaveDeck::Invoke` | ✅ | virtual | Slot [2] of Engine::DeckQuerySaveDeck::vftable, 303 bytes. Creates an action named OpenDialog through FUN_012ec220; writes ScreenName=SaveDeckDialog and DialogName=saveDeckDialog.swf with FUN_012ebde0; calls slot 0x38 o… |
