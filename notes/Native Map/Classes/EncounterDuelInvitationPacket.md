# Class: EncounterDuelInvitationPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a98df0` | `EncounterDuelInvitationPacket::~EncounterDuelInvitationPacket` | 🟢 | dtor | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of sub-opcode 0x68 of the inbound family-41 dispatcher 00aa36c0 and from EH unwind fun… |
| `00a9a200` | `EncounterDuelInvitationPacket::EncounterDuelInvitationPacket` | ✅ | ctor | Calls 008a2fd0 with sub-opcode 0x68, stores EncounterDuelInvitationPacket::vftable, and copies the DAT_01be7b10/DAT_01be7b14 pair into both +0x18/+0x1c and +0x20/+0x24. |
| `00a9a280` | `EncounterDuelInvitationPacket::ScalarDeletingDtor` | ✅ | dtor | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set; returns the object pointer. Occupies slot [0] of Encounter… |
