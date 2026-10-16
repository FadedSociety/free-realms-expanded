# Class: EncounterMatchmaking::SelectQueueForUserPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a99010` | `EncounterMatchmaking::SelectQueueForUserPacket::SelectQueueForUserPacket` | 🟢 | ctor | Calls 00a7f0d0 with sub-opcode 0xc (matchmaking-family base constructor), stores EncounterMatchmaking::SelectQueueForUserPacket::vftable and zeroes the word at +0xc. |
| `00a99070` | `EncounterMatchmaking::SelectQueueForUserPacket::~SelectQueueForUserPacket` | 🟢 | dtor | Stores EncounterMatchmaking::BaseMatchmakingPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of dispatch case 0xc of 00aa2980 and from EH unwind funclet 01… |
| `00a9a3d0` | `EncounterMatchmaking::SelectQueueForUserPacket::ScalarDeletingDtor` | ✅ | dtor | Stores EncounterMatchmaking::BaseMatchmakingPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set; returns the object pointer. Occupi… |
