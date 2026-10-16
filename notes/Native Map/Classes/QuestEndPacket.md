# Class: QuestEndPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c79d20` | `— ⚠reject` | ⚪ | dtor | Destructor body: calls FUN_0103d010 on the object (reward-bundle member teardown) under an SEH frame, then resets the vtable to BasePacket::vftable. |
| `00c7b990` | `QuestEndPacket::ReadFields` | 🟢 | function | Reads the common prefix, an 8-byte value, three 32-bit values, a reward bundle, and a float; short input sets the reader error and NaN progress sets its NaN flag. |
| `00c7bbd0` | `QuestEndPacket::Unserialize` | 🟢 | method | Rejects null packet or buffer, parses through a bounded reader, and reports success only when no reader error occurred and the caller's trailing-byte policy is satisfied. |
| `00c7c030` | `— ⚠reject` | ⚪ | ctor | Calls the base constructor with subtype 13, installs QuestEndPacket::vftable, stores the quest id and four scalar fields, initializes the reward bundle, stores a trailing scalar, and retains an optional pointer. |
| `00c7c0f0` | `QuestEndPacket::DeletingDestruct → QuestEndPacket::ScalarDeletingDtor` | ✅ | dtor | Calls the QuestEndPacket destructor body at 00c79d20, frees the object when flag bit 0 is set, and otherwise returns the object pointer. |
