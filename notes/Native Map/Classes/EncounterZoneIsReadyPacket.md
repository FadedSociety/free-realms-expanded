# Class: EncounterZoneIsReadyPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a98c40` | `EncounterZoneIsReadyPacket::EncounterZoneIsReadyPacket` | ✅ | ctor | Calls 008a2fd0 with sub-opcode 0x6b and stores EncounterZoneIsReadyPacket::vftable; no other field is initialised. |
| `00a98ca0` | `EncounterZoneIsReadyPacket::~EncounterZoneIsReadyPacket` | 🟢 | dtor | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of sub-opcode 0x6b of the inbound family-41 dispatcher 00aa36c0 and from EH unwind fun… |
| `00a9a040` | `EncounterZoneIsReadyPacket::ScalarDeletingDtor` | ✅ | dtor | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set; returns the object pointer. Occupies slot [0] of Encounter… |
