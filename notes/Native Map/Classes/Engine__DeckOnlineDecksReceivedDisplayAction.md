# Class: Engine::DeckOnlineDecksReceivedDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01272880` | `— ⚠reject` | ⚪ | ctor | Returns this. Stores Engine::IDisplayActionHandler::vftable and then Engine::DeckOnlineDecksReceivedDisplayAction::vftable into the object; when the int argument is nonzero it calls FUN_012ea300(this). |
| `01272910` | `Engine::DeckOnlineDecksReceivedDisplayAction::DebugName` | ✅ | virtual | Slot [4] of Engine::DeckOnlineDecksReceivedDisplayAction::vftable. Bytes b8 imm32 c3: returns the constant C string kOnlineDecksReceivedDisplayAction : 0 : ..\common\...\deckhandlers.cc:33. |
| `01272920` | `Engine::DeckOnlineDecksReceivedDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Slot [0] of Engine::DeckOnlineDecksReceivedDisplayAction::vftable. Stores Engine::IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 thunk; retu… |
