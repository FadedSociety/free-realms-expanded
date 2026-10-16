# Class: GuildInviteAcceptPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a09db0` | `— ⚠reject` | ⚪ | ctor | Constructor: runs the guild base constructor with sub-opcode 8, installs GuildInviteAcceptPacket::vftable and stores the two arguments at +0x10 and +0x14. |
| `00a09e20` | `GuildInviteAcceptPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets vtables to BaseGuildPacket and BasePacket and frees this when the delete flag is set. |
| `00c0e8f0` | `GuildInviteAcceptPacket::GuildInviteAcceptPacket ⚠downgrade` | 🟢 | ctor | Constructor: calls the guild invite-packet base constructor FUN_00b17220 and then stores GuildInviteAcceptPacket::vftable at *this (no extra member initialisation); returns the object. |
