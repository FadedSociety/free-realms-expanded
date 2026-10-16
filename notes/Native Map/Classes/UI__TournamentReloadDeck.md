# Class: UI::TournamentReloadDeck

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0123c9d0` | `UI::TournamentReloadDeck::Ctor` | ✅ | ctor | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentReloadDeck::vftable into the object, and when the flag argument is nonzero calls FUN_012ea300 on it. |
| `0123ca60` | `UI::TournamentReloadDeck::GetTraceLabel` | 🟢 | virtual | Virtual slot [4] of UI::TournamentReloadDeck::vftable: returns the address of the literal kReloadDeckDisplayAction label with its source file and line. |
| `0123ca70` | `UI::TournamentReloadDeck::ScalarDeletingDtor` | ✅ | dtor | Stores Engine::IDisplayActionHandler::vftable into the object, then frees it through _free when bit 0 of the flag argument is set. |
