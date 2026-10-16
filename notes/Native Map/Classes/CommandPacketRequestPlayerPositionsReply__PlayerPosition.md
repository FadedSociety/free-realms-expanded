# Class: CommandPacketRequestPlayerPositionsReply::PlayerPosition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a85ae0` | `PlayerPositionDtorBody` | 🟢 | dtor | Destructor body of one PlayerPosition row: rewrites the embedded SoeUtil::IString<char> at this+8 to its static vtable value and, when the string holds a heap buffer (length field at +0x14 > 0), decrements the refcount … |
| `00a85ba0` | `PlayerPositionCtor` | 🟢 | ctor | Default constructor body of a PlayerPosition row: stores the two default dwords from DAT_01be79c0/DAT_01be79c4 at +0 and +4 (the default 64-bit guid), installs the embedded SoeUtil::IString<char> at +8 with the static e… |
| `00a85c20` | `PlayerPositionCtorWrapper` | 🟢 | ctor | SEH-framed constructor wrapper that runs the PlayerPosition default-constructor body 00a85ba0 on the object in ecx and returns that object. |
| `00a85c70` | `PlayerPositionScalarDeletingDtor` | 🟢 | dtor | Deleting destructor wrapper for a PlayerPosition row: runs the destructor body 00a85ae0 and frees the node with _free when the delete flag bit 0 is set. |
