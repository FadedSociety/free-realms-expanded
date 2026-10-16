# Class: Engine::DeckReloadCollectionDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012729e0` | `Engine::DeckReloadCollectionDisplayAction::ctor` | ✅ | ctor | Returns this. Stores Engine::IDisplayActionHandler::vftable and then Engine::DeckReloadCollectionDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `01272a70` | `Engine::DeckReloadCollectionDisplayAction::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string kReloadCollectionDisplayAction : 0 : ..\common\...\deckhandlers.cc:39. |
| `01272a80` | `Engine::DeckReloadCollectionDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Engine::IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `012744f0` | `Engine::DeckReloadCollectionDisplayAction::Process ⚠downgrade` | 🟢 | virtual | Slot [2] of Engine::DeckReloadCollectionDisplayAction::vftable, 30 bytes (decoded: call FUN_012e0ba0; call FUN_0126dad0; if non-null call its vtable slot at +0x18; xor eax,eax; ret 8). Returns 0. |
