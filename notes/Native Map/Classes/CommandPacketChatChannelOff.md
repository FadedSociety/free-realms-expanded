# Class: CommandPacketChatChannelOff

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c1dd60` | `CommandPacketChatChannelOff::Ctor` | ✅ | ctor | Calls the command base constructor with sub-opcode 0x21, installs its packet vptr, initializes an IString member and assigns the supplied channel string. |
| `00c1ddf0` | `CommandPacketChatChannelOff::ScalarDeletingDtor` | ✅ | dtor | Calls embedded string and packet-base cleanup, frees the object when the deleting flag is set, and otherwise returns it. |
