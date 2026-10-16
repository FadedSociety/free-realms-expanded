# Class: QuestFailedPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c79660` | `QuestFailedPacket::Construct ⚠downgrade` | 🟢 | ctor | Constructs a QuestFailedPacket: calls FUN_00a86340(5) (BaseQuestPacket base constructor with subtype 5), stores the QuestFailedPacket vtable and the quest id argument at +0xc. |
| `00c7a200` | `QuestFailedPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BasePacket::vftable and calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `00c7b270` | `QuestFailedPacket::ReadFields` | 🟢 | function | Reads the common prefix and one 32-bit quest id at output+0xc; short input zeroes the value, sets the reader error byte, and clamps the cursor. |
| `00c7b490` | `QuestFailedPacket::Unserialize` | 🟢 | method | Rejects null packet or buffer, creates a bounded reader, parses the quest id, and succeeds only without reader error and with complete consumption unless trailing data is allowed. |
