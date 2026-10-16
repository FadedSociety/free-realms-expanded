# Class: Engine::DeckResetFilter

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01273750` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::DeckResetFilter::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `012737c0` | `Engine::DeckResetFilter::ActionName` | ✅ | virtual | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string ResetFilters. |
| `012737e0` | `Engine::DeckResetFilter::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string ResetFilters : 0 : ..\common\...\deckhandlers.cc:356. |
| `012737f0` | `Engine::DeckResetFilter::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `012757c0` | `Engine::DeckResetFilter::Invoke ⚠downgrade` | 🟢 | virtual | Slot [2] of Engine::DeckResetFilter::vftable, 234 bytes. Gets the thread-owned object; reads the Value field as a flag (FUN_012eb310) and passes it to virtual slot 0x84; creates an action named InitializeDeckCollection … |
