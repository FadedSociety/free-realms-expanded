# Class: UI::TournamentUpdateTournamentTimer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0123cc70` | `UI::TournamentUpdateTournamentTimer::Ctor` | ✅ | ctor | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentUpdateTournamentTimer::vftable into the object, and when the flag argument is nonzero calls FUN_012ea300 on it. |
| `0123cd00` | `UI::TournamentUpdateTournamentTimer::GetTraceLabel` | 🟢 | virtual | Virtual slot [4] of UI::TournamentUpdateTournamentTimer::vftable: returns the address of its diagnostic literal. |
| `0123cd10` | `UI::TournamentUpdateTournamentTimer::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set, after storing the base vtable. |
