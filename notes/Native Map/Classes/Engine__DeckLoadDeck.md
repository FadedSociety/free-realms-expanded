# Class: Engine::DeckLoadDeck

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012743f0` | `Engine::DeckLoadDeck::ctor` | ✅ | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::DeckLoadDeck::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01274460` | `Engine::DeckLoadDeck::ActionName` | ✅ | virtual | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string LoadDeck. |
| `01274480` | `Engine::DeckLoadDeck::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string LoadDeck : 0 : ..\common\...\deckhandlers.cc:63. |
| `01274490` | `Engine::DeckLoadDeck::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `01274610` | `— ⚠reject` | ⚪ | virtual | Slot [2] of Engine::DeckLoadDeck::vftable, 633 bytes. Gets the thread-owned object via FUN_0126dad0; reads the named field DAT_01811404 (FUN_012eb250) and passes it to virtual slot 0x1c, which returns a success byte; cr… |
