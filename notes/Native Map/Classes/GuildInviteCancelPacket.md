# Class: GuildInviteCancelPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b19460` | `GuildInviteCancelPacket::GuildInviteCancelPacket` | ✅ | ctor | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0x1a (26), installs GuildInviteCancelPacket::vftable, initializes the embedded IString at +0x18 (vtable store plus the shared empty-string pointer a… |
| `00b194d0` | `— ⚠reject` | ⚪ | dtor | Destructor body of the invite-cancel packet: rewrites the IString vtable at +0x18, releases the shared string buffer through the IString vtable slot 2 when the reference count is positive, then restores BaseGuildPacket:… |
| `00b1a2b0` | `— ⚠reject` | ⚪ | dtor | Slot [0] scalar deleting destructor of GuildInviteCancelPacket::vftable: calls the destructor body 00b194d0 and frees the object with _free when bit 0 of the flag argument is set, otherwise returns the pointer. |
| `00b1ac50` | `— ⚠reject` | ⚪ | method | Reads the body of an invite-cancel packet: two 16-bit header values through 00b1a810, an 8-byte pair into +0x10/+0x14 and a length-prefixed block skipped through 00894b10, flagging the reader when a field runs past the … |
