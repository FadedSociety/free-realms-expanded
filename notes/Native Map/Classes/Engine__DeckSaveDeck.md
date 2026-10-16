# Class: Engine::DeckSaveDeck

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01272cd0` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::DeckSaveDeck::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01272d40` | `Engine::DeckSaveDeck::ActionName` | ✅ | virtual | Slot [1] of Engine::DeckSaveDeck::vftable. Bytes b8 imm32 c3: returns the constant C string SaveDeck. |
| `01272d60` | `Engine::DeckSaveDeck::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string SaveDeck : 0 : ..\common\...\deckhandlers.cc:89. |
| `01272d70` | `Engine::DeckSaveDeck::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `012748a0` | `Engine::DeckSaveDeck::Invoke ⚠downgrade` | 🟢 | virtual | Slot [2] of Engine::DeckSaveDeck::vftable, 721 bytes. Gets the thread-owned object via FUN_0126dad0; if non-null: when the named field at DAT_01811404 exists passes it (via FUN_012eb250) to virtual slot 0x48 of the obje… |
