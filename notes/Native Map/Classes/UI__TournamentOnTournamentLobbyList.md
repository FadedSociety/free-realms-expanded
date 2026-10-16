# Class: UI::TournamentOnTournamentLobbyList

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0123db90` | `UI::TournamentOnTournamentLobbyList::Ctor` | ✅ | ctor | Stores Scaleform::IScaleformActionHandler::vftable and then UI::TournamentOnTournamentLobbyList::vftable into the object, and when the flag argument is nonzero calls FUN_012ea4f0 on it. |
| `0123dc00` | `UI::TournamentOnTournamentLobbyList::GetActionName` | 🟢 | virtual | Virtual slot [1] of UI::TournamentOnTournamentLobbyList::vftable: returns the address of the literal OnTournamentLobbyList. |
| `0123dc20` | `UI::TournamentOnTournamentLobbyList::GetTraceLabel` | 🟢 | virtual | Virtual slot [4] of UI::TournamentOnTournamentLobbyList::vftable: returns the address of its diagnostic literal. |
| `0123dc30` | `UI::TournamentOnTournamentLobbyList::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
