# Class: EncounterPacketPlayerEnter

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a98a70` | `EncounterPacketPlayerEnter::~EncounterPacketPlayerEnter` | 🟢 | dtor | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. Called from the end of sub-opcode 2 of the inbound family-41 dispatcher 00aa36c0 and from EH unwind funcle… |
| `00a99e10` | `EncounterPacketPlayerEnter::EncounterPacketPlayerEnter` | ✅ | ctor | Calls 008a2fd0 with sub-opcode 2, stores EncounterPacketPlayerEnter::vftable, sets the words at +0x18 and +0x1c from DAT_01be7b10 and DAT_01be7b14 and clears the byte at +0x20. 00a9ccc0 then fills +0x18/+0x1c (8 bytes) … |
| `00a99e80` | `EncounterPacketPlayerEnter::ScalarDeletingDtor` | ✅ | dtor | Stores BaseEncounterPacket::vftable and then BasePacket::vftable into the object, then frees the object via _free (00d82463) when bit 0 of the flag byte is set; returns the object pointer. Occupies slot [0] of Encounter… |
| `00a9ccc0` | `EncounterPacketPlayerEnter::ReadBody` | ✅ | method | Reads the shared 12-byte encounter header through 008d6690 into object+4, +8, +0xc and +0x10, then reads 8 bytes into +0x18/+0x1c under one bound check and 1 byte into +0x20 under a 1-byte bound check, normalising it to… |
