# Class: EncounterInvitationResponsePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a3100` | `EncounterInvitationResponsePacket::~EncounterInvitationResponsePacket` | 🟢 | dtor | Destructor body: restores BaseEncounterPacket::vftable then BasePacket::vftable; 009471a0 destroys the response object with it in a case whose branch first constructs EncounterInvitationResponsePacket (FUN_008b6da0, sub… |
| `008b6da0` | `EncounterInvitationResponsePacket::EncounterInvitationResponsePacket ⚠downgrade` | 🟢 | ctor | Calls the encounter family constructor 008a2fd0(0x67) and installs EncounterInvitationResponsePacket::vftable; no field is initialised. |
| `008b6e00` | `EncounterInvitationResponsePacket::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseEncounterPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
