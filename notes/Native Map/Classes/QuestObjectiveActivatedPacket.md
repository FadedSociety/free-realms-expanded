# Class: QuestObjectiveActivatedPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c798a0` | `QuestObjectiveActivatedPacket::Construct ⚠downgrade` | 🟢 | ctor | Constructs a QuestObjectiveActivatedPacket: calls FUN_00a86340(8) (BaseQuestPacket base constructor with subtype 8), stores the vtable and four fields (quest id at +0xc, objective words at +0x10 and +0x14, byte at +0x18… |
| `00c7a2c0` | `QuestObjectiveActivatedPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BasePacket::vftable and calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `00c7aeb0` | `QuestObjectiveActivatedPacket::ReadFields` | ✅ | function | Reads the common prefix, three 32-bit values at output+0xc/+0x10/+0x14 and a boolean at +0x18; short values are zeroed and flag the reader. |
| `00c7b570` | `QuestObjectiveActivatedPacket::Unserialize` | 🟢 | method | Rejects null inputs, initializes a buffer-bounded reader, invokes the objective-activation field reader, and checks its error and consumption state. |
