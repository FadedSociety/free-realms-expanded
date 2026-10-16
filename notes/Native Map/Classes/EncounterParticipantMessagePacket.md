# Class: EncounterParticipantMessagePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a9b990` | `EncounterParticipantMessagePacket::EncounterParticipantMessagePacket` | ✅ | ctor | Calls 008a2fd0 with sub-opcode 0x78, stores EncounterParticipantMessagePacket::vftable, then builds an inline SoeUtil::IString<char> member: IString<char>::vftable at +0x20, buffer DAT_01be7abc at +0x24, and zeroes the … |
| `00a9ba00` | `EncounterParticipantMessagePacket::~EncounterParticipantMessagePacket` | 🟢 | dtor | Stores the IString<char> vtable at +0x20, releases the string at +0x24 when its count at +0x2c is positive, then stores BaseEncounterPacket::vftable and BasePacket::vftable. Called from EH unwind funclet 015c1c65 and fr… |
| `00a9c400` | `EncounterParticipantMessagePacket::ScalarDeletingDtor` | 🟢 | dtor | Calls 00a9ba00 (the EncounterParticipantMessagePacket destructor body) on this, then if the low bit of the flag byte is set calls _free on the object and returns it. |
