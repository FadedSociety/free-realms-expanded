# Class: UI::TournamentEventSendEventPlayerFlagsDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0123ee20` | `UI::TournamentEventSendEventPlayerFlagsDisplayAction::Ctor` | ✅ | ctor | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentEventSendEventPlayerFlagsDisplayAction::vftable into the object, and when the flag argument is nonzero calls FUN_012ea300 on it. |
| `0123eeb0` | `UI::TournamentEventSendEventPlayerFlagsDisplayAction::GetTraceLabel` | 🟢 | virtual | Virtual slot [4] of UI::TournamentEventSendEventPlayerFlagsDisplayAction::vftable: returns the address of its diagnostic literal. |
| `0123eec0` | `UI::TournamentEventSendEventPlayerFlagsDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set. |
