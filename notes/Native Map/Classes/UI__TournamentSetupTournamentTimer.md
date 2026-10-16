# Class: UI::TournamentSetupTournamentTimer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0123cb20` | `UI::TournamentSetupTournamentTimer::Ctor` | ✅ | ctor | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentSetupTournamentTimer::vftable into the object, and when the flag argument is nonzero calls FUN_012ea300 on it. |
| `0123cbb0` | `UI::TournamentSetupTournamentTimer::GetTraceLabel` | 🟢 | virtual | Virtual slot [4] of UI::TournamentSetupTournamentTimer::vftable: returns the address of its diagnostic literal. |
| `0123cbc0` | `UI::TournamentSetupTournamentTimer::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set, after storing the base vtable. |
