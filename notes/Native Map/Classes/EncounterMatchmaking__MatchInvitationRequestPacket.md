# Class: EncounterMatchmaking::MatchInvitationRequestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a991c0` | `EncounterMatchmaking::MatchInvitationRequestPacket::~MatchInvitationRequestPack…` | 🟢 | dtor | Stores EncounterMatchmaking::BaseMatchmakingPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of dispatch case 9 of 00aa2980 and from EH unwind funclet 015c… |
| `00a9a580` | `EncounterMatchmaking::MatchInvitationRequestPacket::MatchInvitationRequestPacket` | 🟢 | ctor | Calls 00a7f0d0 with sub-opcode 9 (matchmaking-family base constructor) and stores EncounterMatchmaking::MatchInvitationRequestPacket::vftable. |
| `00a9a5e0` | `EncounterMatchmaking::MatchInvitationRequestPacket::ScalarDeletingDtor` | ✅ | dtor | Stores EncounterMatchmaking::BaseMatchmakingPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set; returns the object pointer. Occupi… |
