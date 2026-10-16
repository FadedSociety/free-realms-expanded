# Class: UI::TournamentShowTournamentScores

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0123cdc0` | `UI::TournamentShowTournamentScores::Ctor` | ✅ | ctor | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentShowTournamentScores::vftable into the object, and when the flag argument is nonzero calls FUN_012ea300 on it. |
| `0123ce50` | `UI::TournamentShowTournamentScores::GetTraceLabel` | 🟢 | virtual | Virtual slot [4] of UI::TournamentShowTournamentScores::vftable: returns the address of its diagnostic literal. |
| `0123ce60` | `UI::TournamentShowTournamentScores::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set, after storing the base vtable. |
