# Class: CommandPacketWhoReply

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a99b60` | `CommandPacketWhoReply::CommandPacketWhoReply` | ✅ | ctor | Calls 008a2820 with sub-opcode 2 and zero extra word, stores CommandPacketWhoReply::vftable, and zeroes the words at +0x10, +0x14 and +0x18. |
| `00aa05f0` | `CommandPacketWhoReply::~CommandPacketWhoReply` | ✅ | dtor | Destructor body: rewrites the object vtable to CommandPacketWhoReply::vftable, then frees the player list through FUN_00a9f4d0, and finally rewrites the object vtable to BaseCommandPacket::vftable and then BasePacket::v… |
| `00aa0660` | `CommandPacketWhoReply::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor: calls the destructor body FUN_00aa05f0 and, when bit 0 of the flags argument is set, calls _free on the object; returns the object pointer. |
