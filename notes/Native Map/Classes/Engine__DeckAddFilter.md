# Class: Engine::DeckAddFilter

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012734b0` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::DeckAddFilter::vftable into the object; when the int argument is nonzero it calls FUN_012ea4f0(this). |
| `01273520` | `Engine::DeckAddFilter::ActionName` | ✅ | virtual | Slot [1] of its vftable. Bytes b8 imm32 c3: returns the constant C string AddFilter. |
| `01273540` | `Engine::DeckAddFilter::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string AddFilter : 0 : ..\common\...\deckhandlers.cc:242. |
| `01273550` | `Engine::DeckAddFilter::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `01275320` | `Engine::DeckAddFilter::Invoke` | ✅ | virtual | Slot [2] of Engine::DeckAddFilter::vftable, 607 bytes. Reads PropertyKey and Comparison (FUN_012eb290) and a filter type dword; switch on the type: 0 reads an int Value and calls slot 0x70 with it (value 0x2234b expands… |
