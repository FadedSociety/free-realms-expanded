# Class: CommandPacketChatChannelOn

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c1dcb0` | `CommandPacketChatChannelOn::Ctor` | ✅ | ctor | Calls the command base constructor with sub-opcode 0x20, installs its packet vptr, initializes an IString member and assigns the supplied channel string. |
| `00c1dd40` | `CommandPacketChatChannelOn::ScalarDeletingDtor` | ✅ | dtor | Calls embedded string and packet-base cleanup, frees the object when the deleting flag is set, and otherwise returns it. |
