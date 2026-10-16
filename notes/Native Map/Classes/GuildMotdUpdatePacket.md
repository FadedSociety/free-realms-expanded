# Class: GuildMotdUpdatePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b19560` | `GuildMotdUpdatePacket::GuildMotdUpdatePacket` | ✅ | ctor | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0xb (11), installs GuildMotdUpdatePacket::vftable, zeroes the two dwords at +0x10 and +0x14 and initializes the embedded IString member at +0x18 (vt… |
| `00b195d0` | `— ⚠reject` | ⚪ | dtor | Destructor body of the MOTD packet: rewrites the IString vtable at +0x18, releases the string buffer through IString vtable slot 2 when the reference count is positive, then restores BaseGuildPacket::vftable and BasePac… |
| `00b1a3f0` | `GuildMotdUpdatePacket::ScalarDeletingDtor` | ✅ | dtor | Slot [0] scalar deleting destructor of GuildMotdUpdatePacket::vftable: calls the destructor body 00b195d0 and frees the object with _free when bit 0 of the flag argument is set. |
| `00b1ab20` | `— ⚠reject` | ⚪ | method | Reads the body of a MOTD update: two 16-bit header values through 00b1a810, an 8-byte pair into +0x10/+0x14 and a length-prefixed block skipped through 00894b10, flagging the reader when a field runs past the end. |
