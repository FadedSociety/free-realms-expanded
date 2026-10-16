# Class: Engine::DeckSetSearch

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012739f0` | `Engine::DeckSetSearch::ctor` | ✅ | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::DeckSetSearch::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01273a60` | `Engine::DeckSetSearch::ActionName` | ✅ | virtual | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string SetSearch. |
| `01273a80` | `Engine::DeckSetSearch::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string SetSearch : 0 : ..\common\...\deckhandlers.cc:418. |
| `01273a90` | `Engine::DeckSetSearch::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
