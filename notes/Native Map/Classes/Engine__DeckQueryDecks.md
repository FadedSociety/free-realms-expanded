# Class: Engine::DeckQueryDecks

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01274090` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::DeckQueryDecks::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01274100` | `Engine::DeckQueryDecks::ActionName` | ✅ | virtual | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string QueryDecks. |
| `01274120` | `Engine::DeckQueryDecks::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string QueryDecks : 0 : ..\common\...\deckhandlers.cc:495. |
| `01274130` | `Engine::DeckQueryDecks::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `01276070` | `Engine::DeckQueryDecks::Invoke` | ✅ | virtual | Slot [2] of Engine::DeckQueryDecks::vftable, 269 bytes. Creates an action named QueryDecks through FUN_012ec220; calls slot 0x38 of the thread-owned object for a result object; stores it in DAT_0188ce5c through FUN_012e… |
