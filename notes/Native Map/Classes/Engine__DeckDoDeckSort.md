# Class: Engine::DeckDoDeckSort

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012738a0` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::DeckDoDeckSort::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01273910` | `Engine::DeckDoDeckSort::ActionName` | ✅ | virtual | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string DoDeckSort. |
| `01273930` | `Engine::DeckDoDeckSort::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string DoDeckSort : 0 : ..\common\...\deckhandlers.cc:370. |
| `01273940` | `Engine::DeckDoDeckSort::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `012758b0` | `Engine::DeckDoDeckSort::Invoke` | ✅ | virtual | Slot [2] of Engine::DeckDoDeckSort::vftable, 366 bytes. Reads an integer sort mode (FUN_012eb290 on DAT_017ea910) and calls slot 0x94 of the thread-owned object; for modes 1 to 3 first calls slot 0x90 with (0x1b344, 1);… |
