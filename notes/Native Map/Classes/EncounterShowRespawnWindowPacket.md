# Class: EncounterShowRespawnWindowPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a33b0` | `EncounterShowRespawnWindowPacket::EncounterShowRespawnWindowPacket` | ✅ | ctor | Constructor: calls FUN_008a2fd0(0x7d) so the packet is encounter opcode 41 with sub-opcode 125, installs EncounterShowRespawnWindowPacket::vftable and zeroes param_1[5] and param_1[6] (the two trailing timer fields). |
| `008a3410` | `EncounterShowRespawnWindowPacket::~EncounterShowRespawnWindowPacket` | 🟢 | dtor | Destructor body: restores BaseEncounterPacket::vftable then BasePacket::vftable; it is the last call of the respawn-window case in dispatcher 0090d830 and also runs from EH funclet 0158c118. |
| `008b70a0` | `EncounterShowRespawnWindowPacket::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseEncounterPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
