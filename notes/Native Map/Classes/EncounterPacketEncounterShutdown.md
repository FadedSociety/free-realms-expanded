# Class: EncounterPacketEncounterShutdown

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a98b40` | `EncounterPacketEncounterShutdown::~EncounterPacketEncounterShutdown` | 🟢 | dtor | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of sub-opcode 6 of the inbound family-41 dispatcher 00aa36c0 and from EH unwind funcle… |
| `00a99ef0` | `EncounterPacketEncounterShutdown::EncounterPacketEncounterShutdown` | 🟢 | ctor | Calls 008a2fd0 with sub-opcode 6, stores EncounterPacketEncounterShutdown::vftable, sets the words at +0x18 and +0x1c from DAT_01be7b10 and DAT_01be7b14, and zeroes the words at +0x20 and +0x24. |
| `00a99f60` | `EncounterPacketEncounterShutdown::ScalarDeletingDtor` | ✅ | dtor | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set; returns the object pointer. Occupies slot [0] of Encounter… |
