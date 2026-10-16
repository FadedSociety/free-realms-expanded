# Class: EncounterParticipantResumePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a3250` | `EncounterParticipantResumePacket::EncounterParticipantResumePacket` | ✅ | ctor | Constructor: calls FUN_008a2fd0(0x7a) so the packet is encounter opcode 41 with sub-opcode 122, installs EncounterParticipantResumePacket::vftable and zeroes one byte at param_1+0x14. |
| `008b6fc0` | `EncounterParticipantResumePacket::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseEncounterPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
