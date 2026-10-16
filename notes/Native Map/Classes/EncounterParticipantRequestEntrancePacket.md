# Class: EncounterParticipantRequestEntrancePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b6e70` | `EncounterParticipantRequestEntrancePacket::EncounterParticipantRequestEntranceP…` | 🟢 | ctor | Calls the encounter family constructor 008a2fd0(0x6c), installs EncounterParticipantRequestEntrancePacket::vftable and stores the global pair DAT_01be4850/DAT_01be4854 into +0x18/+0x1c. |
| `008b6ee0` | `EncounterParticipantRequestEntrancePacket::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseEncounterPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
