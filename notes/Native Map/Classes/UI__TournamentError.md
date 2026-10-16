# Class: UI::TournamentError

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0123d0b0` | `UI::TournamentError::Ctor` | ✅ | ctor | Stores Engine::IDisplayActionHandler::vftable and then UI::TournamentError::vftable into the object, and when the flag argument is nonzero calls FUN_012ea300 on it. |
| `0123d140` | `UI::TournamentError::GetTraceLabel` | 🟢 | virtual | Virtual slot [4] of UI::TournamentError::vftable: returns the address of its diagnostic literal. |
| `0123d150` | `UI::TournamentError::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor body: releases the object through _free when bit 0 of the flag argument is set, after storing the base vtable. |
