# Class: CompletedQuestCountUpdatePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c79c50` | `CompletedQuestCountUpdatePacket::Construct` | ✅ | ctor | Constructs a CompletedQuestCountUpdatePacket: calls FUN_00a86340(0xc) (BaseQuestPacket base constructor with subtype 12), stores the vtable and the completed-count argument at +0xc. |
| `00c7a3f0` | `CompletedQuestCountUpdatePacket::ScalarDeletingDtor → CompletedQuestCountUpdate…` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BasePacket::vftable and calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `00c7b300` | `CompletedQuestCountUpdatePacket::ReadFields` | ✅ | function | Reads the common prefix and one 32-bit completed-count field at output+0xc; short input zeroes the field and marks the reader as erroneous. |
| `00c7b730` | `CompletedQuestCountUpdatePacket::Unserialize` | 🟢 | method | Rejects null inputs, constructs a bounded reader, parses the completed-count field and validates reader status and complete consumption policy. |
