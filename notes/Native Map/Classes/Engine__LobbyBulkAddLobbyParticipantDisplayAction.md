# Class: Engine::LobbyBulkAddLobbyParticipantDisplayAction

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0127c700` | `Engine::LobbyBulkAddLobbyParticipantDisplayAction::ctor` | ✅ | ctor | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::LobbyBulkAddLobbyParticipantDisplayAction::vftable into *this; when the int argument is nonzero calls FUN_012ea300(this). |
| `0127c790` | `Engine::LobbyBulkAddLobbyParticipantDisplayAction::DebugName` | ✅ | virtual | Returns the constant C string kBulkAddLobbyParticipantDisplayAction : 0 : ... lobbyhandlers.cc:243 (6 bytes: mov eax, imm32; ret). Virtual slot [4] of its vtable. |
| `0127c7a0` | `Engine::LobbyBulkAddLobbyParticipantDisplayAction::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `0127f8f0` | `— ⚠reject` | ⚪ | virtual | Virtual slot [2]. Returns 1 unless FUN_0127a580() is nonnull. Reads ValueData entry 0 as an integer and entry 1 as an integer list (type 6, else assert mTypeID == kIntegerListTypeID and 0). Scans the list for FUN_012d08… |
