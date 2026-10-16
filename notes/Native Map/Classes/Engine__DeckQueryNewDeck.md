# Class: Engine::DeckQueryNewDeck

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01272e20` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::DeckQueryNewDeck::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01272e90` | `Engine::DeckQueryNewDeck::ActionName` | ✅ | virtual | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string QueryNewDeck. |
| `01272eb0` | `Engine::DeckQueryNewDeck::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string QueryNewDeck : 0 : ..\common\...\deckhandlers.cc:125. |
| `01272ec0` | `Engine::DeckQueryNewDeck::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `01274b80` | `— ⚠reject` | ⚪ | virtual | Slot [2] of Engine::DeckQueryNewDeck::vftable, 659 bytes. Gets the thread-owned object via FUN_0126dad0; when virtual slot 0x24 returns nonzero, forwards the named fields (DAT_01811404, DAT_017ea850, PlayFormat, Flags) … |
