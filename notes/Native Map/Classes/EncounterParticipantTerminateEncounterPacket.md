# Class: EncounterParticipantTerminateEncounterPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a98d40` | `EncounterParticipantTerminateEncounterPacket::EncounterParticipantTerminateEnco…` | ✅ | ctor | Calls 008a2fd0 with sub-opcode 0x79, stores EncounterParticipantTerminateEncounterPacket::vftable and clears the byte at +0x14. |
| `00a98da0` | `EncounterParticipantTerminateEncounterPacket::~EncounterParticipantTerminateEnc…` | 🟢 | dtor | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of sub-opcode 0x79 of the inbound family-41 dispatcher 00aa36c0 and from EH unwind fun… |
| `00a9a190` | `EncounterParticipantTerminateEncounterPacket::ScalarDeletingDtor` | ✅ | dtor | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set; returns the object pointer. Occupies slot [0] of Encounter… |
