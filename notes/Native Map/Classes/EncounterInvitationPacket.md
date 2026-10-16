# Class: EncounterInvitationPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a98cf0` | `EncounterInvitationPacket::~EncounterInvitationPacket` | 🟢 | dtor | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of sub-opcode 0x66 of the inbound family-41 dispatcher 00aa36c0 and from EH unwind fun… |
| `00a9a0b0` | `EncounterInvitationPacket::EncounterInvitationPacket` | ✅ | ctor | Calls 008a2fd0 with sub-opcode 0x66, stores EncounterInvitationPacket::vftable, sets the words at +0x18 and +0x1c from DAT_01be7b10 and DAT_01be7b14, and zeroes +0x20 and +0x24. |
| `00a9a120` | `EncounterInvitationPacket::ScalarDeletingDtor` | ✅ | dtor | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set; returns the object pointer. Occupies slot [0] of Encounter… |
