# Class: QuestAddPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c7ba90` | `QuestAddPacket::Destruct` | 🟢 | dtor | Destroys the embedded quest record by calling FUN_00912450 and restores BasePacket::vftable on the packet object. |
| `00c7bf10` | `QuestAddPacket::Construct` | ✅ | ctor | Calls the base quest-packet constructor with subtype 3, installs QuestAddPacket::vftable, initializes an embedded quest-data record, and clears byte +0x168. |
| `00c7bf80` | `QuestAddPacket::DeletingDestruct` | ✅ | dtor | Calls the embedded-record destructor at 00c7ba90, conditionally frees the packet when flag bit 0 is set, and otherwise returns the object. |
| `00c7cc80` | `QuestAddPacket::ReadFields` | 🟢 | function | Reads the common prefix, decodes the embedded quest record through FUN_0093c840, then reads one boolean into packet+0x168; a missing flag is zeroed and marks reader error. |
| `00c7ccd0` | `QuestAddPacket::Unserialize` | 🟢 | method | Rejects null packet or input, creates a bounded reader, parses the common prefix/full quest record/select flag, then verifies no reader error and the caller's trailing-byte policy. |
