# Class: GuildInviteTimeOutPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a09f70` | `— ⚠reject` | ⚪ | ctor | Constructor: runs the guild base constructor with sub-opcode 10, installs GuildInviteTimeOutPacket::vftable and stores the two arguments at +0x10 and +0x14. |
| `00a09fe0` | `GuildInviteTimeOutPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets vtables to BaseGuildPacket and BasePacket and frees this when the delete flag is set. |
