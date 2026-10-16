# Class: Engine::DeckReloadDeckDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01272b30` | `Engine::DeckReloadDeckDisplayAction::ctor` | ✅ | ctor | Returns this. Stores Engine::IDisplayActionHandler::vftable and then Engine::DeckReloadDeckDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `01272bc0` | `Engine::DeckReloadDeckDisplayAction::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string kReloadDeckDisplayAction : 0 : ..\common\...\deckhandlers.cc:50. |
| `01272bd0` | `Engine::DeckReloadDeckDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Engine::IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `01274510` | `Engine::DeckReloadDeckDisplayAction::Process ⚠downgrade` | 🟢 | virtual | Slot [2] of Engine::DeckReloadDeckDisplayAction::vftable, 30 bytes, identical to 012744f0 (call FUN_012e0ba0; call FUN_0126dad0; if non-null call its vtable slot at +0x18; return 0 with ret 8). |
