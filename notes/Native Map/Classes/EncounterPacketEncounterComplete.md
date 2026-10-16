# Class: EncounterPacketEncounterComplete

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a98ac0` | `EncounterPacketEncounterComplete::~EncounterPacketEncounterComplete` | 🟢 | dtor | Calls 0103d010 three times on three embedded members, then stores BaseEncounterPacket::vftable and BasePacket::vftable into the object. Called from the end of sub-opcode 3 of the inbound family-41 dispatcher 00aa36c0 (w… |
| `00aa1d70` | `EncounterPacketEncounterComplete::EncounterPacketEncounterComplete` | ✅ | ctor | Constructor taking (byte, a, b, c): calls the encounter-family constructor FUN_008a2fd0 with sub-opcode 3, stores the byte argument at +0x14, installs EncounterPacketEncounterComplete::vftable at +0, stores the three re… |
| `00aa1e20` | `EncounterPacketEncounterComplete::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: calls the destructor body FUN_00a98ac0 and, when bit 0 of the flags argument is set, calls _free on the object; returns the object pointer. |
