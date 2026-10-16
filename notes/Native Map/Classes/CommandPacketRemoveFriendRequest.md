# Class: CommandPacketRemoveFriendRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c1db80` | `CommandPacketRemoveFriendRequest::Ctor` | ✅ | ctor | Calls the command base constructor with sub-opcode 0x0f, installs CommandPacketRemoveFriendRequest::vftable, and copies three strings and three dwords from the supplied record through 008ea990; returns this. |
| `00c1dbf0` | `CommandPacketRemoveFriendRequest::ScalarDeletingDtor` | ✅ | dtor | Calls the packet base cleanup, frees the object when the deleting flag is set, and returns the original pointer otherwise. |
