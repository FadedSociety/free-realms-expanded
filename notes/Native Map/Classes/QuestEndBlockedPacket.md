# Class: QuestEndBlockedPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c7a470` | `QuestEndBlockedPacket::Construct` | ✅ | ctor | Constructs a QuestEndBlockedPacket: calls FUN_00a86340(0x10) (BaseQuestPacket base constructor with subtype 16), stores the vtable and the four 32-bit arguments at +0x10, +0x14, +0x18 and +0x1c. |
| `00c7a4f0` | `QuestEndBlockedPacket::ScalarDeletingDtor → QuestEndBlockedPacket::ScalarDeleti…` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BasePacket::vftable and calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `00c7ad30` | `QuestEndBlockedPacket::ReadFields` | 🟢 | function | Reads the shared 16-bit/32-bit prefix and then four 32-bit fields at output+0x10..+0x1c, marking short input and clamping the cursor for missing fields. |
| `00c7b3b0` | `QuestEndBlockedPacket::Unserialize` | 🟢 | method | Builds a bounded reader over the supplied buffer, calls the end-blocked field reader, and returns true only without reader error and with full consumption unless trailing bytes are permitted. |
