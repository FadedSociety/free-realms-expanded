# Class: Engine::DeckQuerySelectDeck

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01273b50` | `Engine::DeckQuerySelectDeck::ctor` | ✅ | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::DeckQuerySelectDeck::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01273bc0` | `Engine::DeckQuerySelectDeck::ActionName` | ✅ | virtual | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string QuerySelectDeck. |
| `01273be0` | `Engine::DeckQuerySelectDeck::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string QuerySelectDeck : 0 : ..\common\...\deckhandlers.cc:423. |
| `01273bf0` | `Engine::DeckQuerySelectDeck::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `01275a40` | `Engine::DeckQuerySelectDeck::Invoke` | ✅ | virtual | Slot [2] of Engine::DeckQuerySelectDeck::vftable, 456 bytes. Creates an action named OpenDialog through FUN_012ec220; writes ScreenName=SelectDeckDialog and DialogName=selectDeckDialog.swf through FUN_012ebde0; calls sl… |
