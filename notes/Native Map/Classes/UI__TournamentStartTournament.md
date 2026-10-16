# Class: UI::TournamentStartTournament

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0123c730` | `UI::TournamentStartTournament::Ctor` | ✅ | ctor | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentStartTournament::vftable into the object, and when the flag argument is nonzero calls FUN_012ea300 on it. |
| `0123c7c0` | `UI::TournamentStartTournament::GetTraceLabel` | 🟢 | virtual | Virtual slot [4] of UI::TournamentStartTournament::vftable: returns the address of the literal kStartTournamentRoundDisplayAction label with its source file and line. |
| `0123c7d0` | `UI::TournamentStartTournament::ScalarDeletingDtor` | ✅ | dtor | Stores Engine::IDisplayActionHandler::vftable into the object, then frees it through _free when bit 0 of the flag argument is set. |
