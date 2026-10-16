# Class: EncounterMatchmaking::MatchmakingServerStatusPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a990c0` | `EncounterMatchmaking::MatchmakingServerStatusPacket::MatchmakingServerStatusPac…` | 🟢 | ctor | Calls 00a7f0d0 with sub-opcode 0xf (matchmaking-family base constructor) and stores EncounterMatchmaking::MatchmakingServerStatusPacket::vftable. |
| `00a99120` | `EncounterMatchmaking::MatchmakingServerStatusPacket::~MatchmakingServerStatusPa…` | 🟢 | dtor | Stores EncounterMatchmaking::BaseMatchmakingPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of dispatch case 0xf of 00aa2980 and from EH unwind funclet 01… |
| `00a9a440` | `EncounterMatchmaking::MatchmakingServerStatusPacket::ScalarDeletingDtor` | ✅ | dtor | Stores EncounterMatchmaking::BaseMatchmakingPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set; returns the object pointer. Occupi… |
