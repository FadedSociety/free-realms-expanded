# Class: Engine::DeckRemoveFilter

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01273600` | `Engine::DeckRemoveFilter::ctor` | ✅ | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::DeckRemoveFilter::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01273670` | `Engine::DeckRemoveFilter::ActionName` | ✅ | virtual | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string RemoveFilter. |
| `01273690` | `Engine::DeckRemoveFilter::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string RemoveFilter : 0 : ..\common\...\deckhandlers.cc:302. |
| `012736a0` | `Engine::DeckRemoveFilter::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `012755a0` | `Engine::DeckRemoveFilter::Invoke` | ✅ | virtual | Slot [2] of Engine::DeckRemoveFilter::vftable, 531 bytes. Reads PropertyKey, Comparison and the filter type; type 0 reads an int Value and calls slot 0x7c (value 0x2234b expands to three slot-0x7c calls with 140000, 0x2… |
