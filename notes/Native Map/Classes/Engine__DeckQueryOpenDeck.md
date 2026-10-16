# Class: Engine::DeckQueryOpenDeck

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01273210` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::DeckQueryOpenDeck::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01273280` | `Engine::DeckQueryOpenDeck::ActionName` | ✅ | virtual | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string QueryOpenDeck. |
| `012732a0` | `Engine::DeckQueryOpenDeck::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string QueryOpenDeck : 0 : ..\common\...\deckhandlers.cc:210. |
| `012732b0` | `Engine::DeckQueryOpenDeck::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `012750a0` | `Engine::DeckQueryOpenDeck::Invoke` | ✅ | virtual | Slot [2] of Engine::DeckQueryOpenDeck::vftable, 303 bytes. Creates an action named OpenDialog through FUN_012ec220; writes ScreenName=LoadDeleteDeckDialog and DialogName=loadDeleteDeckDialog.swf with FUN_012ebde0; calls… |
