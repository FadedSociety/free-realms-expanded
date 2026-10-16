# Class: QuestObjectiveAddedPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c79810` | `QuestObjectiveAddedPacket::Destruct` | 🟢 | dtor | Destructor body: calls FUN_0103d010 on the object (reward-bundle member teardown) under an SEH frame, then resets the vtable to BasePacket::vftable. |
| `00c7bd30` | `QuestObjectiveAddedPacket::Unserialize` | 🟢 | method | Rejects null inputs, initializes a bounded reader, parses the common prefix and one 32-bit field, then decodes an embedded objective definition; checks reader error and trailing-byte policy. |
| `00c7bfa0` | `QuestObjectiveAddedPacket::Construct` | ✅ | ctor | Calls the base constructor with subtype 7, installs QuestObjectiveAddedPacket::vftable, stores the quest id at +0xc and initializes embedded objective data. |
| `00c7c010` | `QuestObjectiveAddedPacket::DeletingDestruct` | ✅ | dtor | Calls the objective-added packet destructor body at 00c79810, frees the packet if flag bit 0 is set, and otherwise returns the object pointer. |
