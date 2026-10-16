# Class: EncounterMatchmaking::QueueStatsRequestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a7fd30` | `EncounterMatchmaking::QueueStatsRequestPacket::QueueStatsRequestPacket` | ✅ | ctor | Constructor: calls the matchmaking packet base constructor 00a7f0d0 with sub-opcode 0xd (13) and installs the QueueStatsRequestPacket vtable. |
| `00a7fd90` | `EncounterMatchmaking::QueueStatsRequestPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: rewrites the vtable pointer to BaseMatchmakingPacket::vftable and then BasePacket::vftable and frees the object when bit 0 of param_2 is set. |
