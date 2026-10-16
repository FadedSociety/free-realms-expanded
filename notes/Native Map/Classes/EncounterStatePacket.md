# Class: EncounterStatePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a98b90` | `EncounterStatePacket::EncounterStatePacket` | ✅ | ctor | Calls 008a2fd0 with sub-opcode 0x6a, then stores EncounterStatePacket::vftable and zeroes the word at +0x14. |
| `00a98bf0` | `EncounterStatePacket::~EncounterStatePacket` | 🟢 | dtor | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of sub-opcode 0x6a of the inbound family-41 dispatcher 00aa36c0 and from EH unwind fun… |
| `00a99fd0` | `EncounterStatePacket::ScalarDeletingDtor` | ✅ | dtor | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set; returns the object pointer. Occupies slot [0] of Encounter… |
