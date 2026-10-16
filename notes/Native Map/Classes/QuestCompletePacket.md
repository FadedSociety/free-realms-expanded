# Class: QuestCompletePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c79590` | `QuestCompletePacket::Construct ⚠downgrade` | 🟢 | ctor | Constructs a QuestCompletePacket: calls FUN_00a86340(4) (BaseQuestPacket base constructor with subtype 4), stores the QuestCompletePacket vtable and the quest id argument at +0xc. |
| `00c7a1a0` | `QuestCompletePacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BasePacket::vftable and calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `00c7b230` | `QuestCompletePacket::ReadFields` | ✅ | function | Reads the common prefix followed by one 32-bit quest id at output+0xc; on a short read it zeroes the field, flags the reader, and clamps the cursor. |
| `00c7b420` | `QuestCompletePacket::Unserialize` | 🟢 | method | Rejects null packet or buffer, initializes a bounded reader, calls the quest-complete field parser, then checks reader error and optionally requires full buffer consumption. |
