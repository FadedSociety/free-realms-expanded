# Class: UI::TournamentTournamentLobbyList

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0123d250` | `UI::TournamentTournamentLobbyList::Ctor` | ✅ | ctor | Stores Scaleform::IScaleformActionHandler::vftable and then UI::TournamentTournamentLobbyList::vftable into the object, and when the flag argument is nonzero calls FUN_012ea4f0 on it. |
| `0123d2c0` | `UI::TournamentTournamentLobbyList::GetActionName` | 🟢 | virtual | Virtual slot [1] of UI::TournamentTournamentLobbyList::vftable: returns the address of the literal TournamentLobbyList. |
| `0123d2e0` | `UI::TournamentTournamentLobbyList::GetTraceLabel` | 🟢 | virtual | Virtual slot [4] of UI::TournamentTournamentLobbyList::vftable: returns the address of its diagnostic literal. |
| `0123d2f0` | `UI::TournamentTournamentLobbyList::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
