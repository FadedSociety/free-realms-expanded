# Class: GuildRoleNameChangedPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b181d0` | `GuildRoleNameChangedPacket::GuildRoleNameChangedPacket ⚠downgrade` | 🟢 | ctor | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0x15 (21), installs GuildRoleNameChangedPacket::vftable and initializes four member dwords at +0x10 through +0x1c to 0,0,0,-1. |
| `00b18240` | `GuildRoleNameChangedPacket::ScalarDeletingDtor` | ✅ | dtor | Slot [0] scalar deleting destructor of GuildRoleNameChangedPacket::vftable: under an exception frame it restores BaseGuildPacket::vftable and then BasePacket::vftable and frees the object with _free when bit 0 of the fl… |
| `00b1aa90` | `GuildRoleNameChangedPacket::ReadBody` | 🟢 | method | Reads the body of a role-rename notification: two 16-bit header values through 00b1a810, an 8-byte pair into +0x10/+0x14 and then two 32-bit values into +0x18 and +0x1c, flagging the reader when a field runs past the en… |
