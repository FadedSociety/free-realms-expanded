# Class: UI::TournamentUpdateTournament

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0123c5e0` | `UI::TournamentUpdateTournament::Ctor` | ✅ | ctor | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentUpdateTournament::vftable into the object, and when the flag argument is nonzero calls FUN_012ea300 on it; returns with ret 4. |
| `0123c670` | `UI::TournamentUpdateTournament::GetTraceLabel` | 🟢 | virtual | Virtual slot [4] of UI::TournamentUpdateTournament::vftable: returns the address of the literal kUpdateTournamentDisplayAction label with its source file and line. |
| `0123c680` | `UI::TournamentUpdateTournament::ScalarDeletingDtor` | ✅ | dtor | Stores Engine::IDisplayActionHandler::vftable into the object (the derived vtable store is absent in this body), then frees the object through _free when bit 0 of the flag argument is set; restores ExceptionList. |
