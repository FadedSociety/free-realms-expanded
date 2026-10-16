# Class: Engine::DeckRequestDeckCollection

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01273ca0` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::DeckRequestDeckCollection::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01273d10` | `Engine::DeckRequestDeckCollection::ActionName` | ✅ | virtual | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string RequestDeckCollection. |
| `01273d30` | `Engine::DeckRequestDeckCollection::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string RequestDeckCollection : 0 : ..\common\...\deckhandlers.cc:443. |
| `01273d40` | `Engine::DeckRequestDeckCollection::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `01275c20` | `Engine::DeckRequestDeckCollection::Invoke` | ✅ | virtual | Slot [2] of Engine::DeckRequestDeckCollection::vftable, 202 bytes. Gets the thread-owned object via FUN_0126dad0, creates an action named InitializeDeckCollection through FUN_012ec220, calls slot 0x44 of the object with… |
