# Class: GuildInviteDeclinePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a09e90` | `— ⚠reject` | ⚪ | ctor | Constructor: runs the guild base constructor with sub-opcode 9, installs GuildInviteDeclinePacket::vftable and stores the two arguments at +0x10 and +0x14. |
| `00a09f00` | `GuildInviteDeclinePacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets vtables to BaseGuildPacket and BasePacket and frees this when the delete flag is set. |
| `00c0e950` | `GuildInviteDeclinePacket::GuildInviteDeclinePacket ⚠downgrade` | 🟢 | ctor | Constructor: calls the guild invite-packet base constructor FUN_00b17220 and then stores GuildInviteDeclinePacket::vftable at *this; returns the object. |
