# Class: GuildPaidRenameCheckReplyPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b19960` | `GuildPaidRenameCheckReplyPacket::GuildPaidRenameCheckReplyPacket ⚠downgrade` | 🟢 | ctor | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0x1e (30), installs GuildPaidRenameCheckReplyPacket::vftable, zeroes the dwords at +0x10 and +0x14, initializes the embedded IString at +0x18 and ze… |
| `00b199e0` | `— ⚠reject` | ⚪ | dtor | Destructor body of the paid rename reply: rewrites the IString vtable at +0x18, releases the string buffer through IString vtable slot 2 when the reference count is positive, then restores BaseGuildPacket::vftable and B… |
| `00b1a470` | `GuildPaidRenameCheckReplyPacket::ScalarDeletingDtor` | ✅ | dtor | Slot [0] scalar deleting destructor of GuildPaidRenameCheckReplyPacket::vftable: calls the destructor body 00b199e0 and frees the object with _free when bit 0 of the flag argument is set. |
| `00b1a900` | `— ⚠reject` | ⚪ | method | Reads the body of a paid-rename reply: two 16-bit header values through 00b1a810, an 8-byte pair into +0x10/+0x14, a length-prefixed block skipped through 00894b10, then a trailing 32-bit value into +0x28, flagging the … |
