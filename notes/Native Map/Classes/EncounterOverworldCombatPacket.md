# Class: EncounterOverworldCombatPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a98e40` | `EncounterOverworldCombatPacket::EncounterOverworldCombatPacket` | ✅ | ctor | Calls 008a2fd0 with sub-opcode 0x84, stores EncounterOverworldCombatPacket::vftable and clears the byte at +0x14. |
| `00a98ea0` | `EncounterOverworldCombatPacket::~EncounterOverworldCombatPacket` | 🟢 | dtor | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of sub-opcode 0x84 of the inbound family-41 dispatcher 00aa36c0 and from EH unwind fun… |
| `00a9a2f0` | `EncounterOverworldCombatPacket::ScalarDeletingDtor` | ✅ | dtor | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set; returns the object pointer. Occupies slot [0] of Encounter… |
