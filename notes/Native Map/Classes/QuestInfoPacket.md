# Class: QuestInfoPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c79470` | `— ⚠reject` | ⚪ | dtor | Destructor body: calls FUN_0103d010 on the object (teardown of the embedded reward-bundle members) under an SEH frame, then resets the vtable to BasePacket::vftable. |
| `00c7b7a0` | `QuestInfoPacket::ReadFields` | 🟢 | function | Reads the common prefix, six 32-bit fields and a boolean, parses an embedded reward bundle, reads an 8-byte value and float, then two booleans; short fields flag reader error and NaN flags the NaN state. |
| `00c7bb60` | `QuestInfoPacket::Unserialize` | 🟢 | method | Rejects null packet or buffer, initializes a bounded reader and calls the full quest-info parser; returns success only with no reader error and allowed buffer consumption. |
| `00c7be00` | `— ⚠reject` | ⚪ | ctor | Calls the base quest-packet constructor with subtype 1, stores scalar arguments and flags, initializes the embedded reward bundle, stores trailing fields, and retains the final non-null reference through FUN_0103a1e0. |
| `00c7bef0` | `QuestInfoPacket::DeletingDestruct` | ✅ | dtor | Calls the QuestInfoPacket destructor body at 00c79470, frees the object only when flag bit 0 is set, then otherwise returns the object pointer. |
