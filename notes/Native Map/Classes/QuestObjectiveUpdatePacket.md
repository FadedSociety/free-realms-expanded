# Class: QuestObjectiveUpdatePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c799c0` | `QuestObjectiveUpdatePacket::Construct` | ✅ | ctor | Constructs a QuestObjectiveUpdatePacket: calls FUN_00a86340(9) (BaseQuestPacket base constructor with subtype 9), stores the vtable and the quest id, objective fields and the byte and dword arguments at +0xc through +0x… |
| `00c7a320` | `QuestObjectiveUpdatePacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BasePacket::vftable and calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `00c7af60` | `QuestObjectiveUpdatePacket::ReadFields` | ✅ | function | Reads the common prefix, three 32-bit values, a float at +0x18, a boolean at +0x1c, and a final 32-bit value at +0x20; NaN sets the reader's NaN flag and short fields set its error flag. |
| `00c7b5e0` | `QuestObjectiveUpdatePacket::Unserialize` | 🟢 | method | Rejects null inputs, sets up a bounded reader, invokes the objective-update body parser, and checks reader errors and the trailing-byte policy. |
