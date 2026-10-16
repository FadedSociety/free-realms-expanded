# Class: QuestObjectiveFailedPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c79b70` | `QuestObjectiveFailedPacket::Construct` | ✅ | ctor | Constructs a QuestObjectiveFailedPacket: calls FUN_00a86340(0xb) (BaseQuestPacket base constructor with subtype 11), stores the vtable and the quest id and objective words at +0xc and +0x10. |
| `00c7a390` | `QuestObjectiveFailedPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BasePacket::vftable and calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `00c7b160` | `QuestObjectiveFailedPacket::ReadFields` | 🟢 | function | Reads the common prefix followed by two 32-bit fields at output+0xc and +0x10; a short field is zeroed, sets the reader error flag and clamps the cursor. |
| `00c7b6c0` | `QuestObjectiveFailedPacket::Unserialize` | 🟢 | method | Rejects null inputs, builds a bounded reader, parses the objective-failure fields, and succeeds only when reader error is clear and consumption meets the trailing-byte policy. |
