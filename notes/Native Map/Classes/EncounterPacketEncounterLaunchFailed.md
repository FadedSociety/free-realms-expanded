# Class: EncounterPacketEncounterLaunchFailed

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a989c0` | `EncounterPacketEncounterLaunchFailed::EncounterPacketEncounterLaunchFailed` | ✅ | ctor | Calls 008a2fd0 with sub-opcode 4 (the shared BaseEncounterPacket constructor), then stores EncounterPacketEncounterLaunchFailed::vftable and zeroes the word at +0x14. |
| `00a98a20` | `EncounterPacketEncounterLaunchFailed::~EncounterPacketEncounterLaunchFailed` | 🟢 | dtor | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of sub-opcode 4 of the inbound family-41 dispatcher 00aa36c0 and from EH unwind funcle… |
| `00a99da0` | `EncounterPacketEncounterLaunchFailed::ScalarDeletingDtor` | ✅ | dtor | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set; returns the object pointer. Occupies slot [0] of Encounter… |
