# Class: BugSubmissionPacketAddBug

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009f9eb0` | `BugSubmissionPacketAddBug::Ctor` | ✅ | ctor | Initializes BaseBugSubmissionPacket with argument 1, installs BugSubmissionPacketAddBug vtable, and initializes two string subobjects with shared empty buffers. |
| `009f9f40` | `BugSubmissionPacketAddBug::Dtor` | 🟢 | dtor | For the string subobject at +0x24, gates cleanup on length +0x30>0; for the one at +0x10, gates on +0x1c>0. Each path decrements its buffer count and calls virtual slot +8 on receiver this+0x24 or this+0x10, respectivel… |
| `009fa330` | `BugSubmissionPacketAddBug::ScalarDeletingDtor` | ✅ | dtor | Calls the packet destructor, frees the object when flag bit 0 is set, and otherwise returns its pointer. |
