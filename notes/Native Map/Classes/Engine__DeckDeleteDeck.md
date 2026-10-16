# Class: Engine::DeckDeleteDeck

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01273df0` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::DeckDeleteDeck::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01273e60` | `Engine::DeckDeleteDeck::ActionName` | ✅ | virtual | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string DeleteDeck. |
| `01273e80` | `Engine::DeckDeleteDeck::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string DeleteDeck : 0 : ..\common\...\deckhandlers.cc:457. |
| `01273e90` | `Engine::DeckDeleteDeck::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `01275cf0` | `— ⚠reject` | ⚪ | virtual | Slot [2] of Engine::DeckDeleteDeck::vftable, 450 bytes. Reads the named field DAT_01811404 as a string and calls slot 0x28 of the thread-owned object with it and an output byte; creates an action named DeleteDeck throug… |
