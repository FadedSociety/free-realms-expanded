# Class: EncounterRunningListResponsePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa3500` | `EncounterRunningListResponsePacket::EncounterRunningListResponsePacket` | ✅ | ctor | Calls FUN_008a2fd0(0x82) (encounter base constructor with sub-opcode 130), stores EncounterRunningListResponsePacket::vftable, installs the EncounterDetailsCommon list vftable at object+0x14 and zeroes the list head, ta… |
| `00aa3570` | `EncounterRunningListResponsePacket::~EncounterRunningListResponsePacket` | 🟢 | dtor | Stores the EncounterDetailsCommon list vftable at object+0x14, unlinks head nodes at +0x18 through FUN_00aa3220, then stores BaseEncounterPacket::vftable and BasePacket::vftable. |
| `00aa3620` | `EncounterRunningListResponsePacket::ScalarDeletingDtor` | ✅ | dtor | Calls FUN_00aa3570 on this, frees this when param_2 bit 0 is set, and otherwise returns this. |
