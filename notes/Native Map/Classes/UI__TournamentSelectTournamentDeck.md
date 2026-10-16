# Class: UI::TournamentSelectTournamentDeck

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0123d4f0` | `UI::TournamentSelectTournamentDeck::Ctor` | ✅ | ctor | Stores Scaleform::IScaleformActionHandler::vftable and then UI::TournamentSelectTournamentDeck::vftable into the object, and when the flag argument is nonzero calls FUN_012ea4f0 on it. |
| `0123d560` | `UI::TournamentSelectTournamentDeck::GetActionName` | 🟢 | virtual | Virtual slot [1] of UI::TournamentSelectTournamentDeck::vftable: returns the address of the literal SelectTournamentDeck. |
| `0123d580` | `UI::TournamentSelectTournamentDeck::GetTraceLabel` | 🟢 | virtual | Virtual slot [4] of UI::TournamentSelectTournamentDeck::vftable: returns the address of its diagnostic literal. |
| `0123d590` | `UI::TournamentSelectTournamentDeck::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
