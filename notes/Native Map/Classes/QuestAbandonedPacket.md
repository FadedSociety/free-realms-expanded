# Class: QuestAbandonedPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c79730` | `QuestAbandonedPacket::Construct` | ✅ | ctor | Constructs a QuestAbandonedPacket: calls FUN_00a86340(6) (BaseQuestPacket base constructor with subtype 6), stores the QuestAbandonedPacket vtable, the quest id at +0xc and the byte argument at +0x10. |
| `00c7a260` | `QuestAbandonedPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BasePacket::vftable and calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `00c7ae40` | `QuestAbandonedPacket::ReadFields` | 🟢 | function | Reads the shared prefix, then a 32-bit quest id at output+0xc and one boolean byte at +0x10; missing values are zeroed and set the reader error flag. |
| `00c7b500` | `QuestAbandonedPacket::Unserialize` | 🟢 | method | Rejects null inputs, creates a bounded reader, invokes the abandoned-packet field reader, and returns success only without reader error and with full consumption unless permitted trailing bytes remain. |
