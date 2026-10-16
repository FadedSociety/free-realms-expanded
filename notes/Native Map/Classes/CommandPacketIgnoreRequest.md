# Class: CommandPacketIgnoreRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c1dc10` | `CommandPacketIgnoreRequest::Ctor` | ✅ | ctor | Calls the command base constructor with sub-opcode 0x1e, installs CommandPacketIgnoreRequest::vftable, copies three strings and three dwords from the supplied record through 008ea990, stores the supplied flag byte at ob… |
| `00c1dc90` | `CommandPacketIgnoreRequest::ScalarDeletingDtor` | ✅ | dtor | Calls the packet base cleanup, frees the object when the deleting flag is set, and returns the original pointer otherwise. |
