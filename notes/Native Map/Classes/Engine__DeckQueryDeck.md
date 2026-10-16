# Class: Engine::DeckQueryDeck

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01273f40` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::DeckQueryDeck::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01273fb0` | `Engine::DeckQueryDeck::ActionName` | ✅ | virtual | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string QueryDeck. |
| `01273fd0` | `Engine::DeckQueryDeck::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string QueryDeck : 0 : ..\common\...\deckhandlers.cc:478. |
| `01273fe0` | `Engine::DeckQueryDeck::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `01275ec0` | `Engine::DeckQueryDeck::Invoke` | ✅ | virtual | Slot [2] of Engine::DeckQueryDeck::vftable, 414 bytes. Reads the named field DAT_01811404 as a string, creates an action named QueryDeck through FUN_012ec220, writes the field back with FUN_012ebe00, calls slot 0x34 of … |
