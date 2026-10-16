# Class: EncounterMatchmaking::RemoveParticipantFromQueueResponsePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a99170` | `RemoveParticipantFromQueueResponsePacket::~RemoveParticipantFromQueueResponsePa…` | 🟢 | dtor | Stores EncounterMatchmaking::BaseMatchmakingPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of dispatch case 0x12 of 00aa2980 and from EH unwind funclet 0… |
| `00a9a4b0` | `RemoveParticipantFromQueueResponsePacket::RemoveParticipantFromQueueResponsePac…` | 🟢 | ctor | Calls 00a7f0d0 with sub-opcode 0x12 (matchmaking-family base constructor) and stores EncounterMatchmaking::RemoveParticipantFromQueueResponsePacket::vftable. |
| `00a9a510` | `EncounterMatchmaking::RemoveParticipantFromQueueResponsePacket::ScalarDeletingD…` | ✅ | dtor | Stores EncounterMatchmaking::BaseMatchmakingPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set; returns the object pointer. Occupi… |
