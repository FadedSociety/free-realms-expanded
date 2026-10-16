# Class: EncounterParticipantCancelPendingEncounterPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a3300` | `EncounterParticipantCancelPendingEncounterPacket` | ✅ | ctor | Constructor: calls FUN_008a2fd0(0x7c) so the packet is encounter opcode 41 with sub-opcode 124, then installs EncounterParticipantCancelPendingEncounterPacket::vftable. |
| `008a3360` | `EncounterParticipantCancelPendingEncounterPacket::Dtor` | 🟢 | dtor | Destructor body: restores BaseEncounterPacket::vftable then BasePacket::vftable; caller 00938fe0 constructs the cancel packet and destroys it with this function. |
| `008b7030` | `EncounterParticipantCancelPendingEncounterPacket::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseEncounterPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
