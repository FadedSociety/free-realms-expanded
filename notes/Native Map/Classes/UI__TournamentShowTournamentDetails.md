# Class: UI::TournamentShowTournamentDetails

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0123d8e0` | `UI::TournamentShowTournamentDetails::Ctor` | ✅ | ctor | Stores Scaleform::IScaleformActionHandler::vftable and then UI::TournamentShowTournamentDetails::vftable into the object, and when the flag argument is nonzero calls FUN_012ea4f0 on it. |
| `0123d950` | `UI::TournamentShowTournamentDetails::GetActionName` | 🟢 | virtual | Virtual slot [1] of UI::TournamentShowTournamentDetails::vftable: returns the address of the literal ShowTournamentDetails. |
| `0123d970` | `UI::TournamentShowTournamentDetails::GetTraceLabel` | 🟢 | virtual | Virtual slot [4] of UI::TournamentShowTournamentDetails::vftable: returns the address of its diagnostic literal. |
| `0123d980` | `UI::TournamentShowTournamentDetails::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
