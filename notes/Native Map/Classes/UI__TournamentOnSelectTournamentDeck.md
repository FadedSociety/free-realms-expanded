# Class: UI::TournamentOnSelectTournamentDeck

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0123de40` | `UI::TournamentOnSelectTournamentDeck::Ctor` | ✅ | ctor | Stores Scaleform::IScaleformActionHandler::vftable and then UI::TournamentOnSelectTournamentDeck::vftable into the object, and when the flag argument is nonzero calls FUN_012ea4f0 on it. |
| `0123deb0` | `UI::TournamentOnSelectTournamentDeck::GetActionName` | 🟢 | virtual | Virtual slot [1] of UI::TournamentOnSelectTournamentDeck::vftable: returns the address of the literal OnSelectTournamentDeck. |
| `0123ded0` | `UI::TournamentOnSelectTournamentDeck::GetTraceLabel` | 🟢 | virtual | Virtual slot [4] of UI::TournamentOnSelectTournamentDeck::vftable: returns the address of its diagnostic literal. |
| `0123dee0` | `UI::TournamentOnSelectTournamentDeck::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
