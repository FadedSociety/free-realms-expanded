# Class: EncounterPacketObjectiveState

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a9c300` | `EncounterPacketObjectiveState::EncounterPacketObjectiveState` | ✅ | ctor | Calls 008a2fd0 with sub-opcode 5, stores EncounterPacketObjectiveState::vftable, then builds an inline SoeUtil::Array<ObjectiveData,0,1> member at +0x14 (vftable) with zeroed buffer, count and capacity at +0x18, +0x1c a… |
| `00a9c370` | `EncounterPacketObjectiveState::~EncounterPacketObjectiveState` | 🟢 | dtor | Stores the Array<ObjectiveData,0,1> vtable at +0x14, calls 009ae540 on the word at +0x1c, releases the buffer at +0x18 through 00401c00 with alignment 8, zeroes the buffer, then stores BaseEncounterPacket::vftable and B… |
| `00a9df10` | `EncounterPacketObjectiveState::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: calls the destructor body FUN_00a9c370 and, when bit 0 of the flags argument is set, calls _free on the object; returns the object pointer. Bytes are 56 8b f1 e8 .. f6 44 24 08 01 74 09 … |
