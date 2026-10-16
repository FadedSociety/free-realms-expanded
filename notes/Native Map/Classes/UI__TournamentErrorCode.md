# Class: UI::TournamentErrorCode

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0123cf10` | `UI::TournamentErrorCode::Ctor` | ✅ | ctor | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentErrorCode::vftable into the object, and when the flag argument is nonzero calls FUN_012ea300 on it. |
| `0123cfa0` | `UI::TournamentErrorCode::GetTraceLabel` | 🟢 | virtual | Virtual slot [4] of UI::TournamentErrorCode::vftable: returns the address of its diagnostic literal. |
| `0123cfb0` | `UI::TournamentErrorCode::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set, after storing the base vtable. |
| `0123d010` | `UI::TournamentErrorCode::ValidateArg0Int` | 🟢 | virtual | Virtual slot [2] of UI::TournamentErrorCode::vftable: calls FUN_012f8f60(0) and, when its type id at +4 equals 2, rechecks argument 0 with the same test and asserts mTypeID == kIntegerTypeID through FUN_012f5a60 on fail… |
