# Class: GuildNameAcceptedPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b19a70` | `GuildNameAcceptedPacket::GuildNameAcceptedPacket` | ✅ | ctor | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0x13 (19), installs GuildNameAcceptedPacket::vftable, zeroes the dwords at +0x10 and +0x14 and initializes the embedded IString member at +0x18. |
| `00b19ae0` | `— ⚠reject` | ⚪ | dtor | Destructor body of the guild name acceptance: rewrites the IString vtable at +0x18, releases the string buffer through IString vtable slot 2 when the reference count is positive, then restores BaseGuildPacket::vftable a… |
| `00b1a490` | `GuildNameAcceptedPacket::ScalarDeletingDtor` | ✅ | dtor | Slot [0] scalar deleting destructor of GuildNameAcceptedPacket::vftable: calls the destructor body 00b19ae0 and frees the object with _free when bit 0 of the flag argument is set. |
| `00b1a870` | `GuildNameAcceptedPacket::ReadBody` | 🟢 | method | Reads the body of a name-accepted guild packet: two 16-bit header values through 00b1a810, then an 8-byte pair into +0x10/+0x14, then a 32-bit length followed by that many bytes skipped through 00894b10, flagging the re… |
