# Class: EncounterMatchmaking::ListQueuesRequestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a7fb90` | `EncounterMatchmaking::ListQueuesRequestPacket::ListQueuesRequestPacket` | ✅ | ctor | Constructor: calls the matchmaking packet base constructor 00a7f0d0 with sub-opcode 1 (that base ctor writes BasePacket::vftable, the base opcode word 0x8d at +4, then BaseMatchmakingPacket::vftable and the sub-opcode a… |
| `00a7fbf0` | `EncounterMatchmaking::ListQueuesRequestPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of the list-queues request packet: rewrites the vtable pointer to BaseMatchmakingPacket::vftable and then BasePacket::vftable, and frees the object with _free when bit 0 of param_2 is set. |
