# Class: GuildDeleteNotificationPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b19660` | `GuildDeleteNotificationPacket::GuildDeleteNotificationPacket` | ✅ | ctor | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0x1d (29), installs GuildDeleteNotificationPacket::vftable, zeroes the two dwords at +0x10 and +0x14 and initializes the embedded IString member at … |
| `00b196d0` | `— ⚠reject` | ⚪ | dtor | Destructor body of the guild-deletion notification: rewrites the IString vtable at +0x18, releases the string buffer through IString vtable slot 2 when the reference count is positive, then restores BaseGuildPacket::vft… |
| `00b1a410` | `GuildDeleteNotificationPacket::ScalarDeletingDtor` | ✅ | dtor | Slot [0] scalar deleting destructor of GuildDeleteNotificationPacket::vftable: calls the destructor body 00b196d0 and frees the object with _free when bit 0 of the flag argument is set. |
| `00b1ace0` | `GuildDeleteNotificationPacket::ReadBody` | 🟢 | method | Reads the body of a guild deletion notification: two 16-bit header values through 00b1a810, an 8-byte pair into +0x10/+0x14 and a length-prefixed block skipped through 00894b10, flagging the reader when a field runs pas… |
