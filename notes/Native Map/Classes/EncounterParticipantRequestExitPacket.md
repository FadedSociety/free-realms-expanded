# Class: EncounterParticipantRequestExitPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a31a0` | `EncounterParticipantRequestExitPacket::EncounterParticipantRequestExitPacket` | ✅ | ctor | Constructor: calls FUN_008a2fd0(0x6d) so the packet is encounter opcode 41 with sub-opcode 109, then installs EncounterParticipantRequestExitPacket::vftable. |
| `008a3200` | `EncounterParticipantRequestExitPacket::~EncounterParticipantRequestExitPacket` | 🟢 | dtor | Destructor body: restores BaseEncounterPacket::vftable then BasePacket::vftable; 009bd240 destroys the request-exit object with it right after constructing the class (sub-opcode 109) and handing it to the gateway, and E… |
| `008b6f50` | `EncounterParticipantRequestExitPacket::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseEncounterPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
