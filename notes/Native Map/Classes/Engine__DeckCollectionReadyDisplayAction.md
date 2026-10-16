# Class: Engine::DeckCollectionReadyDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `012742f0` | `Engine::DeckCollectionReadyDisplayAction::ctor` | ✅ | ctor | Returns this. Stores Engine::IDisplayActionHandler::vftable and then Engine::DeckCollectionReadyDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `01274380` | `Engine::DeckCollectionReadyDisplayAction::DebugName` | ✅ | virtual | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string kCollectionReady : 0 : ..\common\...\deckhandlers.cc:24. |
| `01274390` | `Engine::DeckCollectionReadyDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of its vftable. Stores Engine::IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; returns this. |
| `01274560` | `Engine::DeckCollectionReadyDisplayAction::Process` | ✅ | virtual | Slot [2] of Engine::DeckCollectionReadyDisplayAction::vftable, 165 bytes. Assigns the 24-character string DataFinishedInitializing into a local std::string, passes it to FUN_012ec220 to create an action, dispatches it w… |
