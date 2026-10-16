# Class: QuestObjectiveCompletePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c7a7b0` | `QuestObjectiveCompletePacket::Construct` | ✅ | ctor | Calls the base quest-packet constructor with subtype 10, installs QuestObjectiveCompletePacket::vftable, stores three arguments at +0xc/+0x10/+0x14, zeroes +0x18, and stores the boolean at +0x1c. |
| `00c7a830` | `QuestObjectiveCompletePacket::ScalarDeletingDtor` | ✅ | dtor | Replaces the object's vtable with BasePacket::vftable and frees the object when flag bit 0 is set; otherwise returns the object pointer. |
| `00c7b070` | `QuestObjectiveCompletePacket::ReadFields` | ✅ | function | Reads the common prefix, two 32-bit values, a float, another 32-bit value, and a boolean; NaN progress sets the reader's NaN flag and short reads set its error flag. |
| `00c7b650` | `QuestObjectiveCompletePacket::Unserialize` | 🟢 | method | Rejects null inputs, initializes a bounded reader, invokes the objective-complete field reader, and validates its error and complete-consumption state. |
